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

use serde::{Deserialize, Serialize};

/// Commands sent FROM the phone UI TO the extension via the daemon.
/// The daemon receives these over the /remote WebSocket and forwards
/// them to the extension over the /extension WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    PlayPause,
    Next,
    SeekForward { seconds: f64 },
    SeekBackward { seconds: f64 },
    VolumeUp,
    VolumeDown,
    SetVolume { level: u8 },
}

/// State sent FROM the extension TO the daemon.
/// The daemon receives this and forwards it to all connected phone UIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaState {
    /// Which site is currently active e.g. "netflix", "youtube"
    pub site: String,

    /// Whether media is currently playing
    pub is_playing: bool,

    /// Title of the current content
    pub title: String,

    /// Current playback position in seconds
    pub current_time: f64,

    /// Total duration in seconds
    pub duration: f64,

    /// Volume level 0-100
    pub volume: u8,
}

impl Default for MediaState {
    fn default() -> Self {
        Self {
            site: "unknown".to_string(),
            is_playing: false,
            title: "Nothing playing".to_string(),
            current_time: 0.0,
            duration: 0.0,
            volume: 50,
        }
    }
}
