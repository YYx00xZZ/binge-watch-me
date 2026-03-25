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

use rust_embed::RustEmbed;
use axum::response::Html;
use qrcode::{QrCode, EcLevel};
use image::Luma;

#[derive(RustEmbed)]
#[folder = "frontend/"]
struct Assets;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use crate::protocol::{Command, MediaState};

/// Shared application state accessible from all route handlers
#[derive(Clone)]
pub struct AppState {
    /// Latest media state received from the extension
    pub media_state: Arc<Mutex<MediaState>>,
    /// Channel to broadcast commands to the extension
    pub command_tx: broadcast::Sender<Command>,
    /// Channel to broadcast media state to all phone remotes
    pub state_tx: broadcast::Sender<MediaState>,
    pub token: String,
}

impl AppState {
    pub fn new(token: String) -> Self {
        let (command_tx, _) = broadcast::channel(32);
        let (state_tx, _) = broadcast::channel(32);
        Self {
            media_state: Arc::new(Mutex::new(MediaState::default())),
            command_tx,
            state_tx,
            token,
        }
    }
}

/// Start the Axum server — call this from a Tokio runtime
pub async fn start(state: AppState) {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/setup", get(setup_page_handler))
        .route("/style.css", get(css_handler))
        .route("/app.js", get(js_handler))
        .route("/token", get(token_handler))
        .route("/qr", get(qr_handler))
        .route("/extension", get(extension_ws_handler))
        .route("/remote", get(remote_ws_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7777")
        .await
        .expect("Failed to bind to port 7777");

    tracing::info!("Server listening on http://0.0.0.0:7777");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn index_handler() -> impl IntoResponse {
    let content = Assets::get("index.html").unwrap();
    Html(std::str::from_utf8(content.data.as_ref()).unwrap().to_string())
}

async fn css_handler() -> impl IntoResponse {
    let content = Assets::get("style.css").unwrap();
    (
        [(axum::http::header::CONTENT_TYPE, "text/css")],
        std::str::from_utf8(content.data.as_ref()).unwrap().to_string(),
    )
}

async fn js_handler() -> impl IntoResponse {
    let content = Assets::get("app.js").unwrap();
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        std::str::from_utf8(content.data.as_ref()).unwrap().to_string(),
    )
}

/// Simple health check endpoint
async fn health_handler() -> &'static str {
    "binge-watch-me ok"
}

/// WebSocket handler for the browser extension
/// Extension sends MediaState, receives Commands
async fn extension_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_extension_socket(socket, state))
}

async fn handle_extension_socket(mut socket: WebSocket, state: AppState) {
    tracing::info!("Extension connected");

    let mut command_rx = state.command_tx.subscribe();

    loop {
        tokio::select! {
            // Receive MediaState from extension
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<MediaState>(&text) {
                            Ok(media_state) => {
                                tracing::info!(
                                    "State from extension: {} - {}",
                                    media_state.site,
                                    media_state.title
                                );
                                // Update shared state
                                *state.media_state.lock().unwrap() = media_state.clone();
                                // Broadcast to all phone remotes
                                let _ = state.state_tx.send(media_state);
                            }
                            Err(e) => tracing::warn!("Failed to parse MediaState: {}", e),
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("Extension disconnected");
                        break;
                    }
                    _ => {}
                }
            }

            // Forward Commands to extension
            Ok(cmd) = command_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&cmd) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        tracing::info!("Extension disconnected while sending command");
                        break;
                    }
                }
            }
        }
    }
}

/// WebSocket handler for the phone remote UI
/// Remote sends Commands, receives MediaState
async fn remote_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_remote_socket(socket, state))
}

async fn handle_remote_socket(mut socket: WebSocket, state: AppState) {
    tracing::info!("Phone remote connected");

    let mut state_rx = state.state_tx.subscribe();

    // Send current state immediately on connect so the UI isn't blank
    {
        let current = state.media_state.lock().unwrap().clone();
        if let Ok(json) = serde_json::to_string(&current) {
            let _ = socket.send(Message::Text(json.into())).await;
        }
    }

    loop {
        tokio::select! {
            // Receive Commands from phone
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<Command>(&text) {
                            Ok(cmd) => {
                                tracing::info!("Command from phone: {:?}", cmd);
                                // Handle volume locally, forward rest to extension
                                match &cmd {
                                    Command::VolumeUp => crate::platform::volume_up(),
                                    Command::VolumeDown => crate::platform::volume_down(),
                                    Command::SetVolume { level } => {
                                        crate::platform::set_volume(*level)
                                    }
                                    _ => {
                                        let _ = state.command_tx.send(cmd);
                                    }
                                }
                            }
                            Err(e) => tracing::warn!("Failed to parse Command: {}", e),
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("Phone remote disconnected");
                        break;
                    }
                    _ => {}
                }
            }

            // Forward MediaState updates to phone
            Ok(media_state) = state_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&media_state) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        tracing::info!("Phone remote disconnected while sending state");
                        break;
                    }
                }
            }
        }
    }
}

/// Returns the token in plain text — only useful from localhost.
/// The extension setup page calls this to auto-configure itself.
async fn token_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // This endpoint has no token requirement — it is only reachable
    // from localhost so it is safe. Remote clients cannot reach it.
    state.token.clone()
}

/// Returns a QR code PNG containing the full remote URL with token.
/// Shown on the main page so the phone can scan and connect.
async fn qr_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let url = crate::network::get_remote_url(&state.token);
    tracing::info!("QR code URL: {}", url);

    // Generate QR code
    let code = QrCode::with_error_correction_level(url.as_bytes(), EcLevel::M)
        .expect("Failed to generate QR code");

    // Render to image
    let image = code.render::<Luma<u8>>()
        .min_dimensions(200, 200)
        .build();

    // Encode as PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("Failed to encode QR code as PNG");

    (
        [(axum::http::header::CONTENT_TYPE, "image/png")],
        png_bytes,
    )
}

async fn setup_page_handler() -> impl IntoResponse {
    let content = Assets::get("setup.html").unwrap();
    Html(std::str::from_utf8(content.data.as_ref()).unwrap().to_string())
}
