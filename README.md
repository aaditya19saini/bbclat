# bbclat

A lightweight battery monitor for **Linux** and **Windows**, written in Rust. It runs in the background, checks the battery every 10 seconds, and sends a desktop notification when the battery crosses a threshold.

## Features

- Checks battery percentage and charger state every 10 seconds
- Alerts when charging reaches 95%
- Alerts when the battery drops to 20% or below while unplugged
- Sends a more urgent alert at 15% or below while unplugged
- Doesn't repeat an alert until the battery moves back across its threshold
- Tiny footprint: no dependencies on Linux (~350 KB binary, ~2 MB RAM), ~5-6 MB working set on Windows

| | Linux | Windows |
|---|---|---|
| Battery source | `/sys/class/power_supply/BAT*` + AC adapter (`type = Mains`) | `GetSystemPowerStatus` |
| Notifications | `notify-send` (mako, dunst, swaync, Omarchy, GNOME, KDE…) | Tray balloon / notification center |
| Background run | systemd user service | Task Scheduler at logon |

On Linux only the laptop battery (`BAT*`) is read, so wireless mice and other peripherals don't trigger alerts.

## Project layout

```text
src/main.rs        Shared alert logic and thresholds
src/linux.rs       Linux battery reading (sysfs) and notify-send notifications
src/windows.rs     Windows battery reading (Win32) and tray notifications
contrib/bbclat.service   systemd user unit for Linux
build.bat, build2.bat    Windows release build using the VS 2022 environment
install_vc.bat     Installs the Visual C++ tools workload on Windows
```

The thresholds and polling interval are at the top of `src/main.rs`: `CHECK_INTERVAL_SECS`, `HIGH_THRESHOLD`, `LOW_THRESHOLD`, `CRITICAL_LOW_THRESHOLD`.

---

## Linux

### Requirements

- A battery exposed under `/sys/class/power_supply`
- `notify-send` (package `libnotify` on Arch) and a running notification daemon
- Rust (`cargo`)

### Build

```sh
cargo build --release
./target/release/bbclat --status   # prints the current reading and exits
```

### Install as a background service

```sh
install -Dm755 target/release/bbclat ~/.local/bin/bbclat
install -Dm644 contrib/bbclat.service ~/.config/systemd/user/bbclat.service
systemctl --user daemon-reload
systemctl --user enable --now bbclat
```

The service starts with your graphical session and restarts automatically if it exits.

### Manage

```sh
systemctl --user status bbclat
systemctl --user restart bbclat     # e.g. after installing a new build
systemctl --user stop bbclat
journalctl --user -u bbclat
```

### Uninstall

```sh
systemctl --user disable --now bbclat
rm ~/.config/systemd/user/bbclat.service ~/.local/bin/bbclat
systemctl --user daemon-reload
```

---

## Windows

### Requirements

- Rust with the MSVC toolchain
- Visual Studio 2022 C++ build tools with the Windows SDK

If the MSVC build tools are missing, run `install_vc.bat` from an elevated command prompt. It uses the Visual Studio Installer to add the C++ tools workload to Visual Studio 2022 Community. If Visual Studio is installed somewhere else, update the paths in the batch scripts.

### Build

```powershell
cargo build --release
```

The executable is created at `target\release\bbclat.exe`. If Cargo isn't on your `PATH`, run `build.bat` or `build2.bat`, which set up the VS 2022 x64 environment and call Cargo from `%USERPROFILE%\.cargo\bin`.

### Run

The binary has no console window, so launching it just starts a silent background process:

```powershell
.\target\release\bbclat.exe
```

Stop it from Task Manager, or with `Stop-Process -Name bbclat`.

### Run at login

Create a Task Scheduler task that starts bbclat at logon and restarts it if it crashes:

```powershell
$exePath = "$PWD\target\release\bbclat.exe"
$action = New-ScheduledTaskAction -Execute $exePath
$trigger = New-ScheduledTaskTrigger -AtLogOn -User "$env:USERDOMAIN\$env:USERNAME"
$settings = New-ScheduledTaskSettingsSet -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew
Register-ScheduledTask -TaskName "bbclat-battery-monitor" -Action $action -Trigger $trigger -Settings $settings -Description "bbclat: lightweight battery threshold monitor" -Force
Start-ScheduledTask -TaskName "bbclat-battery-monitor"
```

Manage it with `Get-ScheduledTask -TaskName "bbclat-battery-monitor"`, `Stop-ScheduledTask` / `Start-ScheduledTask`, or remove it with `Unregister-ScheduledTask -TaskName "bbclat-battery-monitor"`.

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
