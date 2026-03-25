const WS_URL = `ws://${location.host}/remote`;
const RECONNECT_DELAY_MS = 3000;

let ws = null;
let reconnectTimer = null;
let isPlaying = false;

// DOM refs
const statusEl       = document.getElementById("status");
const statusTextEl   = document.getElementById("status-text");
const titleEl        = document.getElementById("title");
const siteBadgeEl    = document.getElementById("site-badge");
const currentTimeEl  = document.getElementById("current-time");
const durationEl     = document.getElementById("duration");
const progressFillEl = document.getElementById("progress-fill");
const volumeFillEl   = document.getElementById("volume-fill");
const volumeLabelEl  = document.getElementById("volume-label");
const btnPlayPause   = document.getElementById("btn-playpause");

// Connect to daemon WebSocket
function connect() {
  ws = new WebSocket(WS_URL);

  ws.onopen = () => {
    setStatus(true);
    clearTimeout(reconnectTimer);
  };

  ws.onmessage = ({ data }) => {
    try {
      const state = JSON.parse(data);
      updateUI(state);
    } catch (e) {
      console.error("Failed to parse state:", e);
    }
  };

  ws.onclose = () => {
    setStatus(false);
    reconnectTimer = setTimeout(connect, RECONNECT_DELAY_MS);
  };

  ws.onerror = () => ws.close();
}

// Send a command to the daemon
function send(action, extra = {}) {
  if (ws?.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ action, ...extra }));
  }
}

// Update UI from MediaState
function updateUI(state) {
  isPlaying = state.is_playing;

  titleEl.textContent      = state.title;
  siteBadgeEl.textContent  = state.site;
  btnPlayPause.textContent = state.is_playing ? "Pause" : "Play";

  // Progress
  const pct = state.duration > 0
    ? (state.current_time / state.duration) * 100
    : 0;
  progressFillEl.style.width = `${pct}%`;
  currentTimeEl.textContent  = formatTime(state.current_time);
  durationEl.textContent     = formatTime(state.duration);

  // Volume
  volumeFillEl.style.width  = `${state.volume}%`;
  volumeLabelEl.textContent = `${state.volume}%`;
}

function setStatus(connected) {
  statusEl.className        = `status ${connected ? "connected" : "disconnected"}`;
  statusTextEl.textContent  = connected ? "Connected" : "Reconnecting...";
}

function formatTime(seconds) {
  if (!seconds || isNaN(seconds)) return "0:00";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60).toString().padStart(2, "0");
  return `${m}:${s}`;
}

// Button handlers
document.getElementById("btn-playpause")   .addEventListener("click", () => send("play_pause"));
// document.getElementById("btn-seek-back")   .addEventListener("click", () => send("seek_backward", { seconds: 10 }));
// document.getElementById("btn-seek-forward").addEventListener("click", () => send("seek_forward",  { seconds: 10 }));
document.getElementById("btn-next")        .addEventListener("click", () => send("next"));
document.getElementById("btn-vol-down")    .addEventListener("click", () => send("volume_down"));
document.getElementById("btn-vol-up")      .addEventListener("click", () => send("volume_up"));

connect();
