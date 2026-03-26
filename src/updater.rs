// binge-watch-me — a self-hosted media remote controlled from your phone
// Copyright (C) 2026  Aleksandar Parvanov
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::sync::{Arc, Mutex};

const REPO_OWNER: &str = "YYx00xZZ";
const REPO_NAME: &str = "binge-watch-me";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(target_arch = "aarch64")]
const ASSET_NAME: &str = "BingeWatchMe-macos-arm64.zip";
#[cfg(target_arch = "x86_64")]
const ASSET_NAME: &str = "BingeWatchMe-macos-x86.zip";

#[derive(Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
}

/// Check GitHub releases for a newer version than the running binary.
/// Returns None if up-to-date or if the check fails.
pub fn check_for_update() -> Option<UpdateInfo> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()
        .ok()?
        .fetch()
        .ok()?;

    let latest = releases.into_iter().next()?;
    // GitHub tags may have a leading 'v'; strip it for semver comparison
    let version = latest.version.trim_start_matches('v').to_string();

    match self_update::version::bump_is_greater(CURRENT_VERSION, &version) {
        Ok(true) => {}
        _ => return None,
    }

    // Verify the expected asset exists in this release
    if !latest.assets.iter().any(|a| a.name == ASSET_NAME) {
        tracing::warn!("Release v{} has no asset named {}", version, ASSET_NAME);
        return None;
    }

    // self_update stores asset["url"] (the GitHub API URL), which requires
    // an Accept: application/octet-stream header to download binary content.
    // Instead, construct the browser_download_url which works without extra headers.
    let download_url = format!(
        "https://github.com/{}/{}/releases/download/v{}/{}",
        REPO_OWNER, REPO_NAME, version, ASSET_NAME
    );

    Some(UpdateInfo {
        version,
        download_url,
    })
}

/// Runs on a Tokio task: check once after startup, then every 24 hours.
/// Writes an UpdateInfo into `slot` when a newer version is found.
pub async fn start_update_checker(slot: Arc<Mutex<Option<UpdateInfo>>>) {
    // Wait for the server to settle before the first check
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    loop {
        tracing::info!("Checking for updates (current: {})...", CURRENT_VERSION);

        let result = tokio::task::spawn_blocking(check_for_update).await;

        match result {
            Ok(Some(info)) => {
                tracing::info!("Update available: v{}", info.version);
                *slot.lock().unwrap() = Some(info);
                // Stop checking once an update is found
                return;
            }
            Ok(None) => {
                tracing::info!("Up to date");
            }
            Err(e) => {
                tracing::warn!("Update check task failed: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(24 * 3600)).await;
    }
}

/// Download, extract, and install an update via a sidecar shell script.
/// Spawns a background thread and returns immediately.
/// The spawned thread calls std::process::exit(0) when handoff is ready.
pub fn install_update(info: UpdateInfo) {
    std::thread::spawn(move || {
        if let Err(e) = do_install(info) {
            tracing::error!("Update installation failed: {}", e);
        }
    });
}

fn do_install(info: UpdateInfo) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Downloading v{}...", info.version);

    // Scratch directory that persists until the sidecar script copies from it
    let tmp_dir = std::env::temp_dir().join(format!("bwm-update-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir)?;

    // Download the zip via curl — uses the macOS system TLS stack and follows
    // GitHub's CDN redirects reliably. self_update::Download uses rustls with
    // bundled WebPKI roots which can fail on the objects.githubusercontent.com
    // redirect even when the URL works fine in a browser.
    let zip_path = tmp_dir.join("update.zip");
    let curl_status = std::process::Command::new("curl")
        .args([
            "-fsSL",                       // fail on HTTP errors, silent, follow redirects
            "-o", zip_path.to_str().unwrap(),
            &info.download_url,
        ])
        .status()?;
    if !curl_status.success() {
        return Err(format!("curl exited with status {}", curl_status).into());
    }

    tracing::info!("Download complete, extracting...");

    // Use the system unzip rather than self_update's built-in extractor.
    // self_update's zip extraction tries to File::create every entry including
    // directory entries and symlinks, which fails for .app bundles on macOS.
    let unzip_status = std::process::Command::new("unzip")
        .args([
            "-o",                              // overwrite without prompting
            zip_path.to_str().unwrap(),
            "-d",
            tmp_dir.to_str().unwrap(),
        ])
        .status()?;
    if !unzip_status.success() {
        return Err(format!("unzip exited with status {}", unzip_status).into());
    }

    let new_app = tmp_dir.join("BingeWatchMe.app");
    if !new_app.exists() {
        return Err("BingeWatchMe.app not found in zip".into());
    }

    // Strip macOS quarantine flag so the relaunched app isn't blocked
    std::process::Command::new("xattr")
        .args(["-cr", new_app.to_str().unwrap()])
        .status()?;

    // Locate the running .app bundle: exe is at
    //   BingeWatchMe.app/Contents/MacOS/binge-watch-me
    let exe = std::env::current_exe()?;
    let maybe_app = exe
        .parent() // MacOS/
        .and_then(|p| p.parent()) // Contents/
        .and_then(|p| p.parent()) // BingeWatchMe.app/
        .filter(|p| p.extension().map(|e| e == "app").unwrap_or(false))
        .map(|p| p.to_path_buf());

    match maybe_app {
        Some(current_app) => {
            // Running from a proper .app bundle — replace it via sidecar script.
            tracing::info!("Will replace '{}' with '{}'", current_app.display(), new_app.display());

            // Write sidecar script:
            //   $1 = PID of this process
            //   $2 = new .app path (source)
            //   $3 = current .app path (destination)
            let script_path =
                std::env::temp_dir().join(format!("bwm-updater-{}.sh", std::process::id()));
            std::fs::write(
                &script_path,
                "#!/bin/bash\n\
                 while kill -0 \"$1\" 2>/dev/null; do sleep 0.1; done\n\
                 rm -rf \"$3\"\n\
                 cp -R \"$2\" \"$3\"\n\
                 open \"$3\"\n\
                 rm -- \"$0\"\n",
            )?;

            std::process::Command::new("chmod")
                .args(["+x", script_path.to_str().unwrap()])
                .status()?;

            std::process::Command::new("/bin/bash")
                .args([
                    script_path.to_str().unwrap(),
                    &std::process::id().to_string(),
                    new_app.to_str().unwrap(),
                    current_app.to_str().unwrap(),
                ])
                .spawn()?;

            tracing::info!("Sidecar launched — exiting for update handoff");
        }
        None => {
            // Running via `cargo run` or outside a bundle — just open the new
            // .app directly without trying to replace anything.
            tracing::info!(
                "Not running from a .app bundle (dev mode) — opening '{}' directly",
                new_app.display()
            );
            std::process::Command::new("open")
                .arg(&new_app)
                .spawn()?;
        }
    }

    std::process::exit(0);
}
