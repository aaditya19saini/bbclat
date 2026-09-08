# Battery Monitor

A lightweight Windows battery monitor written in Rust. It checks the current battery percentage and charging state once per minute, prints the result to the console, and alerts when the battery reaches a configured threshold.

## Features

- Reads battery status through the Windows `GetSystemPowerStatus` API
- Checks battery status every 60 seconds
- Alerts when charging reaches 95%
- Alerts when discharging reaches 20%
- Prints alerts and plays a console beep
- Avoids repeating the same alert until the battery moves back across its threshold

## Requirements

- Windows
- Rust with the MSVC toolchain
- Visual Studio 2022 C++ build tools with the Windows SDK

The project uses the `windows`, `serde`, and `serde_json` crates. Dependencies are restored automatically by Cargo.

## Build

From the project directory, run:

```powershell
cargo build --release
```

The release executable is created at:

```text
target\release\battery-monitor.exe
```

If Cargo is not available on your `PATH`, the included scripts call the Cargo executable from `%USERPROFILE%\.cargo\bin`:

```bat
build.bat
```

or:

```bat
build2.bat
```

The scripts initialize the Visual Studio 2022 x64 build environment before compiling.

## Run

Run the release build from PowerShell:

```powershell
.\target\release\battery-monitor.exe
```

Keep the process running in a console window while you want battery monitoring enabled. Stop it with `Ctrl+C`.

Example output:

```text
Battery monitor started. Checking every 60 seconds...
Battery: 87% | Charging: true
```

## Alerts

When an alert is triggered, the program writes a message to the console and emits a console bell:

```text
*** NOTIFICATION ***
Title: Battery at 95%
Body: Consider unplugging the charger to preserve battery health.
********************
```

The thresholds and polling interval are defined near the top of `src/main.rs`:

- `CHECK_INTERVAL_SECS`: polling interval, currently 60 seconds
- `HIGH_THRESHOLD`: charging alert threshold, currently 95%
- `LOW_THRESHOLD`: low-battery alert threshold, currently 20%

## Installing Visual Studio C++ Tools

If the MSVC build tools are missing, run `install_vc.bat` from an elevated command prompt. It uses the Visual Studio Installer to add the C++ tools workload to Visual Studio 2022 Community.

If Visual Studio is installed somewhere else, update the paths in the batch scripts before using them.

## Project Layout

```text
src/main.rs       Application entry point and battery monitoring logic
Cargo.toml        Rust package metadata and dependencies
build.bat         Release build using the VS 2022 Community environment
build2.bat        Alternate release build environment setup
install_vc.bat    Installs the Visual C++ tools workload
```

## License

No license has been specified yet.
