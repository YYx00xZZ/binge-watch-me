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

use keyring::Entry;

const SERVICE: &str = "binge-watch-me";
const ACCOUNT: &str = "remote-token";

/// Load the token from the keychain or generate a new one if it doesn't exist.
/// The token is a random 32-character hex string.
pub fn get_or_create_token() -> String {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .expect("Failed to access keychain");

    match entry.get_password() {
        Ok(token) => {
            tracing::info!("Loaded token from keychain");
            token
        }
        Err(_) => {
            let token = generate_token();
            entry.set_password(&token)
                .expect("Failed to store token in keychain");
            tracing::info!("Generated and stored new token in keychain");
            token
        }
    }
}

/// Delete the token from the keychain — forces a new one to be generated
/// on next launch. Useful for a "Reset" option in the tray menu.
pub fn reset_token() {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .expect("Failed to access keychain");
    let _ = entry.delete_credential();
    tracing::info!("Token reset");
}

fn generate_token() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        .hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    let h1 = hasher.finish();

    let mut hasher2 = DefaultHasher::new();
    h1.hash(&mut hasher2);
    std::time::Instant::now().hash(&mut hasher2);
    let h2 = hasher2.finish();

    format!("{:016x}{:016x}", h1, h2)
}
