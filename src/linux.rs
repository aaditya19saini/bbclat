use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::Alert;

const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Returns (battery percent, plugged in). Only looks at the laptop battery
/// (`BAT*`), so peripherals like wireless mice are ignored.
pub fn get_battery_info() -> Result<(u8, bool), String> {
    let entries = fs::read_dir(POWER_SUPPLY_DIR).map_err(|e| e.to_string())?;

    let mut battery: Option<(u8, String)> = None;
    let mut mains_seen = false;
    let mut mains_online = false;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let kind = read_trimmed(&path.join("type")).unwrap_or_default();

        if kind == "Mains" {
            mains_seen = true;
            if read_trimmed(&path.join("online")).as_deref() == Some("1") {
                mains_online = true;
            }
        } else if kind == "Battery" && name.starts_with("BAT") && battery.is_none() {
            let capacity = read_trimmed(&path.join("capacity"))
                .and_then(|s| s.parse::<u16>().ok())
                .map(|c| c.min(100) as u8);
            if let Some(capacity) = capacity {
                let status = read_trimmed(&path.join("status")).unwrap_or_default();
                battery = Some((capacity, status));
            }
        }
    }

    let (percent, status) = battery.ok_or("no battery found")?;
    let plugged_in = if mains_seen {
        mains_online
    } else {
        matches!(status.as_str(), "Charging" | "Full" | "Not charging")
    };
    Ok((percent, plugged_in))
}

/// Sends notifications through `notify-send`.
pub struct Notifier;

impl Notifier {
    pub fn new() -> Option<Self> {
        Some(Notifier)
    }

    pub fn tick(&self) {}

    pub fn notify(&self, title: &str, body: &str, alert: Alert) {
        let (urgency, icon) = match alert {
            Alert::High => ("normal", "battery-full-charging"),
            Alert::Low => ("normal", "battery-low"),
            Alert::Critical => ("critical", "battery-caution"),
        };
        let _ = Command::new("notify-send")
            .args(["-a", "bbclat", "-u", urgency, "-i", icon, title, body])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
