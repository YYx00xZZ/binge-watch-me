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

mod auth;
mod network;
mod platform;
mod protocol;
mod server;
mod tray;
mod updater;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("binge-watch-me starting...");

    let token = auth::get_or_create_token();
    tracing::info!("Token ready");

    let app_state = server::AppState::new(token);

    tray::run(|| {
        tracing::info!("Tray ready — spawning background services...");

        let state = app_state.clone();

        std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .expect("Failed to create Tokio runtime")
                .block_on(server::start(state));
        });

        // Give the server a moment to start then open the setup page
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(500));
            std::process::Command::new("open")
                .arg("http://127.0.0.1:7777/setup")
                .spawn()
                .ok();
        });
    });
}
