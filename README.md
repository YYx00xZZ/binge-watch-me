# Binge Watch Me

Lightweight Flask server that exposes playback and volume controls for Netflix running in Brave on macOS. The server serves a simple web remote (`index.html`) and bridges button presses to AppleScript shortcuts.

## Requirements
- The terminal that is running the script must have Accessabillity permissions
- Brave Browser signed into Netflix
- Conda (or Mamba) to manage the Python environment
- You should enable AppleScript in Brave from Developer tab

You can make it work in other ways too, the project is very simple.

## Setup
1. Create the environment: `conda env create --prefix ./env -f environment.yml`
   - To use a named environment instead, drop the `prefix` line in `environment.yml` and pass `--name <env-name>`.
2. Activate it: `conda activate ./env`

## Run
1. Start the Flask server: `python media_server.py`
2. Open the controller UI in a browser on the same network: `http://<host-ip>:5000/`
   - The page calls REST endpoints like `/playpause`, `/next`, `/netflix/next`, `/prev`, `/volume/up`, and `/volume/down`.
   - Hitting those endpoints directly from other clients works too (e.g., home automation scripts).

## Notes
- The automation scripts assume Brave has the active Netflix tab.

## Build
run pyinstaller BingeWatchMe.spec
run ./dist/BingeWatchMe.app/Contents/MacOS/BingeWatchMe
create .dmg hdiutil create -volname MyApp -srcfolder dist/MyApp.app -ov -format UDZO MyApp.dmg
