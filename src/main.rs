use std::thread;
use std::time::Duration;
use windows::Win32::System::Power::{
    GetSystemPowerStatus, SYSTEM_POWER_STATUS,
};

const CHECK_INTERVAL_SECS: u64 = 60;
const HIGH_THRESHOLD: u8 = 95;
const LOW_THRESHOLD: u8 = 20;

fn main() {
    println!("bbclat battery monitor started. Checking every {} seconds...", CHECK_INTERVAL_SECS);
    
    let mut last_high_notified = false;
    let mut last_low_notified = false;

    loop {
        match get_battery_info() {
            Ok((percent, charging)) => {
                println!("Battery: {}% | Charging: {}", percent, charging);
                
                if charging && percent >= HIGH_THRESHOLD && !last_high_notified {
                    notify("Battery at 95%", "Consider unplugging the charger to preserve battery health.");
                    last_high_notified = true;
                    last_low_notified = false;
                } else if !charging && percent <= LOW_THRESHOLD && !last_low_notified {
                    notify("Battery at 20%", "Plug in the charger.");
                    last_low_notified = true;
                    last_high_notified = false;
                } else if charging && percent < HIGH_THRESHOLD {
                    last_high_notified = false;
                } else if !charging && percent > LOW_THRESHOLD {
                    last_low_notified = false;
                }
            }
            Err(e) => eprintln!("Failed to get battery info: {}", e),
        }
        
        thread::sleep(Duration::from_secs(CHECK_INTERVAL_SECS));
    }
}

fn get_battery_info() -> Result<(u8, bool), String> {
    let mut status = SYSTEM_POWER_STATUS::default();
    let result = unsafe { GetSystemPowerStatus(&mut status) };
    if result.is_err() {
        return Err("GetSystemPowerStatus failed".into());
    }
    
    let percent = status.BatteryLifePercent;
    if percent == 255 {
        return Err("Battery percentage unknown".into());
    }
    
    let charging = status.ACLineStatus == 1; // 1 = online
    Ok((percent, charging))
}

fn notify(title: &str, body: &str) {
    println!("\n*** NOTIFICATION ***");
    println!("Title: {}", title);
    println!("Body: {}", body);
    println!("********************\n");
    
    // Play a simple beep using console
    print!("\x07");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}