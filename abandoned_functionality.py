# ------ Brave next
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

@app.route("/prev")
def prev_track():
    brave_prev()
    return "Previous sent to Brave"
# ------ END Brave next
