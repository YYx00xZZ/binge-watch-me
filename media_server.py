from flask import Flask, send_from_directory
import subprocess

app = Flask(__name__)

# Functions to control Brave/Netflix
def brave_playpause():
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

def brave_next():
    subprocess.run([
        "osascript", "-e",
        '''
        tell application "Brave Browser"
            activate
            tell application "System Events"
                key code 45 using shift down -- 45 is 'N', shift makes it Shift+N
            end tell
        end tell
        '''
    ])

# Previous Episode → Shift+P
def brave_prev():
    subprocess.run([
        "osascript", "-e",
        '''
        tell application "Brave Browser"
            activate
            tell application "System Events"
                key code 35 using shift down -- 35 = 'P'
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
            set volume output volume (currentVolume + 10)
        end if
        '''
    ])

def volume_down():
    subprocess.run([
        "osascript", "-e",
        '''
        set currentVolume to output volume of (get volume settings)
        if currentVolume > 0 then
            set volume output volume (currentVolume - 10)
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

@app.route("/next")
def next_track():
    brave_next()
    return "Next sent to Brave"

@app.route("/prev")
def prev_track():
    brave_prev()
    return "Previous sent to Brave"

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
