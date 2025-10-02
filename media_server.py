import pystray
import threading
import subprocess
import webbrowser
from PIL import Image
from flask import Flask, render_template

app = Flask(__name__)

# Brave + Netflix controls

def brave_focus():
    subprocess.run([
        "osascript", "-e",
        '''
        tell application "System Events"
            if (name of processes) contains "Brave Browser" then
                tell application "Brave Browser" to activate
            else
                tell application "Brave Browser" to launch
                delay 1
                tell application "Brave Browser" to activate
            end if
        end tell
        '''
    ])

def brave_neflix_show_media_controls():
    """
    Brings up Netflix media controls in Brave on macOS by simulating a key press.

    This uses AppleScript via `osascript` to send the F15 key (key code 113),
    which is typically unmapped and safe. Netflix treats any key press as
    user activity and will display its media control overlay without affecting
    playback. 

    Notes:
        - Requires Accessibility and Automation permissions for the terminal/Python.
        - Logs are not printed in this function; only the simulated key press runs.
        - For more detailed logging with timestamps, wrap the call and log around it.
    """
    subprocess.run([
        "osascript", "-e",
        '''
        tell application "System Events" to key code 113
        '''
    ])

# Functions to control Brave/Netflix
def brave_playpause():
    brave_focus()
    subprocess.run([
        "osascript", "-e",
        '''
        tell application "Brave Browser"
            activate
            tell application "System Events"
                keystroke space
            end tell
        end tell
        '''
    ])

# # Netflix specific
def brave_netflix_next():
    brave_focus()
    brave_neflix_show_media_controls()
    subprocess.run([
        "osascript", "-e",
        '''
        tell application "Brave Browser"
            tell front window to tell active tab
                execute javascript "var nextBtn = document.querySelector('button[data-uia=\\\"control-next\\\"]'); if(nextBtn) { nextBtn.click(); }"
            end tell
        end tell
        '''
    ])

# Volume Controls
def volume_up():
    subprocess.run([
        "osascript", "-e",
        '''
        set currentVolume to output volume of (get volume settings)
        if currentVolume < 100 then
            set volume output volume (currentVolume + 5)
        end if
        '''
    ])

def volume_down():
    subprocess.run([
        "osascript", "-e",
        '''
        set currentVolume to output volume of (get volume settings)
        if currentVolume > 0 then
            set volume output volume (currentVolume - 5)
        end if
        '''
    ])

def brave_netflix_seek_backward_10():
    """
    Simulates pressing the Left Arrow key on macOS.
    """

    subprocess.run([
        "osascript", "-e",
        '''
        tell application "System Events"
            key code 123 -- Left Arrow
        end tell
        '''
    ])

def brave_netflix_seek_forward_10():
    """
    Simulates pressing the Right Arrow key on macOS.
    """

    subprocess.run([
        "osascript", "-e",
        '''
        tell application "System Events"
            key code 124 -- Right Arrow
        end tell
        '''
    ])

@app.route("/")
def index():
    return render_template("index.html")

@app.route("/playpause")
def playpause():
    brave_playpause()
    return "Play/Pause sent to Brave"

@app.route("/netflix/next")
def next_netflix_track():
    brave_netflix_next()
    return "Next sent to Brave"

@app.route("/volume/up")
def vol_up():
    volume_up()
    return "Volume increased"

@app.route("/volume/down")
def vol_down():
    volume_down()
    return "Volume decreased"

@app.route("/seek/backward/10")
def seek_backward_10():
    brave_focus()
    brave_neflix_show_media_controls()
    brave_netflix_seek_backward_10()
    return "-10s"

@app.route("/seek/forward/10")
def seek_forward_10():
    brave_focus()
    brave_neflix_show_media_controls()
    brave_netflix_seek_forward_10()
    return "+10s"

# Flask runner

def run_flask():
    app.run(host="0.0.0.0", port=5000, debug=False)

# Tray icon integration

def on_open(icon, item):
    webbrowser.open("http://127.0.0.1:5000")

def on_quit(icon, item):
    icon.stop()

if __name__ == "__main__":
    # Start Flask in background
    flask_thread = threading.Thread(target=run_flask, daemon=True)
    flask_thread.start()

    # Load tray icon image (PNG, e.g. 16x16 or 32x32 transparent)
    icon_image = Image.open("static/images/controller_white.png")

    # Create system tray menu
    icon = pystray.Icon(
        "Netflix Controller",
        icon_image,
        "Netflix Remote",
        menu=pystray.Menu(
            pystray.MenuItem("Open Web UI", on_open),
            pystray.MenuItem("Quit", on_quit)
        )
    )

    # Run the tray loop
    icon.run()
