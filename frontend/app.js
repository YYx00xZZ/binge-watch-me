const params = new URLSearchParams(window.location.search);
const token  = params.get("token");

if (!token) {
  // No token — show a simple message, this page should only be
  // opened via QR code which includes the token automatically
  document.getElementById("app").innerHTML = `
    <div style="text-align:center;padding:40px;color:#888;font-family:sans-serif">
      <p>Open this page by scanning the QR code from the app.</p>
    </div>
  `;
} else {
  startRemote(token);
}

function startRemote(token) {
  const WS_URL = `ws://${window.location.host}/remote?token=${token}`;
  const RECONNECT_DELAY_MS = 3000;

  let ws = null;
  let reconnectTimer = null;

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

  function connect() {
    ws = new WebSocket(WS_URL);
    ws.onopen  = () => { setStatus(true);  clearTimeout(reconnectTimer); };
    ws.onclose = () => { setStatus(false); reconnectTimer = setTimeout(connect, RECONNECT_DELAY_MS); };
    ws.onerror = () => ws.close();
    ws.onmessage = ({ data }) => {
      try { updateUI(JSON.parse(data)); }
      catch (e) { console.error("Failed to parse state:", e); }
    };
  }

  function send(action, extra = {}) {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action, ...extra }));
    }
  }

  function updateUI(state) {
    titleEl.textContent      = state.title;
    siteBadgeEl.textContent  = state.site;
    btnPlayPause.textContent = state.is_playing ? "Pause" : "Play";

    const pct = state.duration > 0
      ? (state.current_time / state.duration) * 100
      : 0;
    progressFillEl.style.width = `${pct}%`;
    currentTimeEl.textContent  = formatTime(state.current_time);
    durationEl.textContent     = formatTime(state.duration);
    volumeFillEl.style.width   = `${state.volume}%`;
    volumeLabelEl.textContent  = `${state.volume}%`;
  }

  function setStatus(connected) {
    statusEl.className       = `status ${connected ? "connected" : "disconnected"}`;
    statusTextEl.textContent = connected ? "Connected" : "Reconnecting...";
  }

  function formatTime(seconds) {
    if (!seconds || isNaN(seconds)) return "0:00";
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60).toString().padStart(2, "0");
    return `${m}:${s}`;
  }

  document.getElementById("btn-playpause").addEventListener("click", () => send("play_pause"));
  document.getElementById("btn-next")     .addEventListener("click", () => send("next"));
  document.getElementById("btn-vol-down") .addEventListener("click", () => send("volume_down"));
  document.getElementById("btn-vol-up")   .addEventListener("click", () => send("volume_up"));

  connect();
}