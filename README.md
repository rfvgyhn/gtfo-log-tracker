# gtfo-log-tracker
Application for the game [GTFO] that reports terminal log progress for the
_D-Lock Block Decipherer_ achievement.

![screenshot]

## Installation

This application doesn't require any installation. It's a standalone executable.

1. Download the [latest release] for your operating system
2. Extract 
   * Windows - right click file and select extract
   * Linux - use your systems archive manager or `tar`
3. Run the executable named _gtfo-log-tracker_

## Features
* **Automatic Tracking**

  The app can both fetch data from PlayFab (GTFO's official storage location for 
  achievement data) and parse your log files to get your progress.

  Using PlayFab will provide the most correct results, but you won't 
  be able to run the tracker at the same time as you play the game due to Steam 
  limitations. You can use the `--playfab` argument to fetch data from PlayFab.

  Parsing from your log files can be less accurate. The game only fetches achievement
  progress when the game starts. Since logs are counted as read for the whole team if one 
  person in your team reads them and the game only writes to your log file when you read
  them, the app won't see an update if a teammate reads a log until you start the game
  again.
  
  Parsing from log files can also provide false positives since some log names are
  shared between different log files. For example, _2MD-N3H-SYH_ is the name of the
  log in _R7D1 205_ and _R8C1 249_ even though they are different logs.

  The default is to read from your log files so the app can be run while you are playing.

* **Auto-filter**

  The app can be set to automatically show only the logs in the level you're 
  currently playing for easier tracking.

## Usage
The application will load the latest log file in your game's data directory. 
It will also watch that directory for changes so you can leave the app open as 
you play and it will automatically update.

You may use the filter text box to narrow down the terminal logs that are shown. 
For example, typing _R1_ into the textbox will only show logs with _R1_ in any 
of the columns.

### Arguments

| Argument    | Effect                                                                                                                                 |
|-------------|----------------------------------------------------------------------------------------------------------------------------------------|
| --playfab   | Get achievement progress from Play Fab                                                                                                 |
| --data-path | Manually specify your GTFO data path if it can't automatically be found (`C:\Users\user\AppData\LocalLow\10 Chambers Collective\GTFO`) |

#### Applying Arguments

* You can apply arguments via your terminal emulator of choice (gnome-terminal, alacritty, CMD, Windows Terminal, etc...)
  
  `/path/to/gtfo-log-tracker --playfab`
* On Windows, you can create a shortcut and append the argument in the _Target_ textbox
  1. Right-click `gtfo-log-tracker.exe`
  2. Select _Create Shortcut_
  3. Right-click newly created shortcut
  4. Select _Properties_
  5. Select _Shortcut_ tab
  6. Add your argument(s) to the _Target_ textbox
  
      ![target-example]

### Troubleshooting
Debug logging is placed in the standard log location for your operating system:
* Windows - `%LOCALAPPDATA%\gtfo-log-tracker\log.txt`
* Linux - `$XDG_STATE_HOME/gtfo-log-tracker/log.txt` (`~/.local/state` if `$XDG_STATE_HOME` isn't set)

## Build
1. [Install Rust]
2. Compile and run the binary:
    ```
    $ git clone https://github.com/rfvgyhn/gtfo-log-tracker
    $ cd gtfo-log-tracker
    $ cargo build --release
    $ cp target/release/build/steamworks-sys-*/out/*steam_api* target/release
    $ ./target/release/gtfo-log-tracker
    ```

[GTFO]: https://store.steampowered.com/app/493520/GTFO/
[latest release]: https://github.com/rfvgyhn/gtfo-log-tracker/releases
[install rust]: https://www.rust-lang.org/tools/install
[screenshot]: https://rfvgyhn.blob.core.windows.net/images/gtfo-log-tracker.webp
[target-example]: https://rfvgyhn.blob.core.windows.net/images/gtfo-log-tracker-windows-shortcut.png