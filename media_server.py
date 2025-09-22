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

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=True)
