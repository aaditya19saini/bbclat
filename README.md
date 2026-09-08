# bbclat

A lightweight Windows battery monitor written in Rust. It runs invisibly in the background (no console window), checks the current battery percentage and charging state every 10 seconds, and alerts when the battery reaches a configured threshold. Working set is ~5-6 MB.

## Features

- Reads battery status through the Windows `GetSystemPowerStatus` API
- Checks battery status every 10 seconds
- Alerts when charging reaches 95%
- Alerts when battery drops to 20% or below while discharging
- Sends a more urgent alert at 15% or below while discharging
- Displays Windows notifications in the notification center
- Runs as a background process with no console window
- Avoids repeating the same alert until the battery moves back across its threshold

## Requirements

- Windows
- Rust with the MSVC toolchain
- Visual Studio 2022 C++ build tools with the Windows SDK

The project uses the `windows` crate. Dependencies are restored automatically by Cargo.

## Build

From the project directory, run:

```powershell
cargo build --release
```

The release executable is created at:

```text
target\release\bbclat.exe
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

The binary has no console window (it runs as a `windows` subsystem app), so launching it just starts a silent background process:

```powershell
.\target\release\bbclat.exe
```

Stop it from Task Manager, or with:

```powershell
Stop-Process -Name bbclat
```

### Running persistently at login

A Windows Task Scheduler task named `bbclat-battery-monitor` is set up to launch the release binary automatically at logon and restart it if it ever crashes. To (re)create it:

```powershell
$exePath = "$PWD\target\release\bbclat.exe"
$action = New-ScheduledTaskAction -Execute $exePath
$trigger = New-ScheduledTaskTrigger -AtLogOn -User "$env:USERDOMAIN\$env:USERNAME"
$settings = New-ScheduledTaskSettingsSet -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew
Register-ScheduledTask -TaskName "bbclat-battery-monitor" -Action $action -Trigger $trigger -Settings $settings -Description "bbclat: lightweight Windows battery threshold monitor" -Force
Start-ScheduledTask -TaskName "bbclat-battery-monitor"
```

Manage it with `Get-ScheduledTask -TaskName "bbclat-battery-monitor"`, `Stop-ScheduledTask`/`Start-ScheduledTask`, or `Unregister-ScheduledTask -TaskName "bbclat-battery-monitor"` to remove it.

## Alerts

When an alert is triggered, the program shows a Windows toast notification. For example, at the low-battery threshold:

```text
Title: Battery at 20%
Body: Plug in the charger.
```

The thresholds and polling interval are defined near the top of `src/main.rs`:

- `CHECK_INTERVAL_SECS`: polling interval, currently 10 seconds
- `HIGH_THRESHOLD`: charging alert threshold, currently 95%
- `LOW_THRESHOLD`: low-battery alert threshold, currently 20%
- `CRITICAL_LOW_THRESHOLD`: critical low-battery alert threshold, currently 15%

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

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
