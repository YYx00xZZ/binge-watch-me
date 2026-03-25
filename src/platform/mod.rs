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

#[cfg(target_os = "macos")]
mod macos;

/// Increase system volume by 5 points
pub fn volume_up() {
    #[cfg(target_os = "macos")]
    macos::volume_up();
}

/// Decrease system volume by 5 points
pub fn volume_down() {
    #[cfg(target_os = "macos")]
    macos::volume_down();
}

/// Set system volume to a specific level (0-100)
pub fn set_volume(level: u8) {
    #[cfg(target_os = "macos")]
    macos::set_volume(level);
}
