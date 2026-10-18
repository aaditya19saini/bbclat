#![cfg_attr(windows, windows_subsystem = "windows")]

use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;

#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

#[cfg(not(any(target_os = "linux", windows)))]
compile_error!("bbclat only supports Linux and Windows");

const CHECK_INTERVAL_SECS: u64 = 10;
const HIGH_THRESHOLD: u8 = 95;
const LOW_THRESHOLD: u8 = 20;
const CRITICAL_LOW_THRESHOLD: u8 = 15;

#[derive(Clone, Copy)]
pub enum Alert {
    High,
    Low,
    Critical,
}

fn main() {
    // `bbclat --status` prints the current reading once and exits.
    // (Linux only in practice: the Windows build has no console to print to.)
    if std::env::args().nth(1).as_deref() == Some("--status") {
        match platform::get_battery_info() {
            Ok((percent, plugged_in)) => println!("battery {}%, plugged in: {}", percent, plugged_in),
            Err(e) => println!("error: {}", e),
        }
        return;
    }

    let Some(notifier) = platform::Notifier::new() else {
        return;
    };

    let mut last_high_notified = false;
    let mut last_low_notified = false;
    let mut last_critical_notified = false;

    loop {
        notifier.tick();

        if let Ok((percent, charging)) = platform::get_battery_info() {
            if charging && percent >= HIGH_THRESHOLD && !last_high_notified {
                notifier.notify(&format!("Battery at {}%", percent), "Consider unplugging the charger to preserve battery health.", Alert::High);
                last_high_notified = true;
                last_low_notified = false;
                last_critical_notified = false;
            } else if !charging && percent <= CRITICAL_LOW_THRESHOLD && !last_critical_notified {
                notifier.notify(&format!("Battery at {}%", percent), "plug your charger, you dumbass", Alert::Critical);
                last_critical_notified = true;
                last_low_notified = true;
                last_high_notified = false;
            } else if !charging && percent <= LOW_THRESHOLD && !last_low_notified {
                notifier.notify(&format!("Battery at {}%", percent), "Plug in the charger.", Alert::Low);
                last_low_notified = true;
                last_high_notified = false;
            } else if charging && percent < HIGH_THRESHOLD {
                last_high_notified = false;
            } else if !charging && percent > LOW_THRESHOLD {
                last_low_notified = false;
                last_critical_notified = false;
            } else if !charging && percent > CRITICAL_LOW_THRESHOLD {
                last_critical_notified = false;
            }
        }

        thread::sleep(Duration::from_secs(CHECK_INTERVAL_SECS));
    }
}
