# Binge Watch Me

Lightweight Flask server that exposes playback and volume controls. For now, it works with Netflix running in Brave on macOS. The server serves a simple web remote (`index.html`) and bridges button presses to AppleScript shortcuts.

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
Because we are poor developers, we can't afford to sign the app. Steps on how to run it:

### From source
1. Get the code
2. Install dependencies
3. Start the app: `python media_server.py`
4. Go to MacOS Settings -> Privacy & Security -> Accessability and allow your terminal or IDE
5. In Brave, enable View -> Developer -> Allow JavaScript from AppleEvents
6. The app also require access to Automation under Settings -> Privacy & Security -> Automation. This is usually given automatically but you can check just to be sure

### From release
1. Download the executable from [the release section of the project](https://github.com/YYx00xZZ/binge-watch-me/releases)
2. Move it to Applications folder
3. While holding Shift, right click the app and hit Open

   You will see the following warning message:
   ```
   macOS cannot verify the developer of “BingeWatchMe”. Are you sure you want to open it?
   ```
   if you want to run it, click Yes.
4. Go to MacOS Settings -> Privacy & Security -> Accessability and allow BingeWatchMe
5. In Brave, enable View -> Developer -> Allow JavaScript from AppleEvents
6. The app also require access to Automation under Settings -> Privacy & Security -> Automation. This is usually given automatically but you can check just to be sure

## Notes
- The automation scripts assume Brave has the active Netflix tab.

## Build
run pyinstaller BingeWatchMe.spec
run ./dist/BingeWatchMe.app/Contents/MacOS/BingeWatchMe
create .dmg hdiutil create -volname MyApp -srcfolder dist/MyApp.app -ov -format UDZO MyApp.dmg

# Disclaimer
This project is not affiliated with, endorsed by, or associated with Netflix, Brave, or any other third party mentioned. All trademarks are the property of their respective owners.