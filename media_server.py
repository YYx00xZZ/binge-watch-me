import time
import subprocess
from flask import Flask, send_from_directory

app = Flask(__name__)

# Function to focus Brave
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

@app.route("/")
def index():
    return send_from_directory(".", "index.html")

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

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=True)
