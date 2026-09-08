use std::thread;
use std::time::Duration;
use windows::Win32::System::Power::{
    GetSystemPowerStatus, SYSTEM_POWER_STATUS,
};
use windows::UI::Notifications::{
    ToastNotificationManager, ToastNotification, ToastTemplateType,
};
use windows::Data::Xml::Dom::XmlDocument;
use windows::core::HSTRING;

const CHECK_INTERVAL_SECS: u64 = 10;
const HIGH_THRESHOLD: u8 = 95;
const LOW_THRESHOLD: u8 = 20;

fn main() {
    println!("Battery monitor started. Checking every {} seconds...", CHECK_INTERVAL_SECS);
    
    let mut last_high_notified = false;
    let mut last_low_notified = false;

    loop {
        match get_battery_info() {
            Ok((percent, charging)) => {
                println!("Battery: {}% | Charging: {}", percent, charging);
                
                if charging && percent >= HIGH_THRESHOLD && !last_high_notified {
                    notify(&format!("Battery at {}%", percent), "Consider unplugging the charger to preserve battery health.");
                    last_high_notified = true;
                    last_low_notified = false;
                } else if !charging && percent <= LOW_THRESHOLD && !last_low_notified {
                    notify(&format!("Battery at {}%", percent), "Plug in the charger.");
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
    
    // Console beep
    print!("\x07");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    
    // Windows Toast Notification
    show_toast(title, body);
}

fn show_toast(title: &str, body: &str) {
    let xml = unsafe { 
        ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02) 
    };
    let xml: XmlDocument = match xml {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Failed to get toast template: {}", e);
            return;
        }
    };
    
    let texts = match xml.GetElementsByTagName(&HSTRING::from("text")) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to get text elements: {}", e);
            return;
        }
    };
    
    let length = match texts.Length() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to get length: {}", e);
            return;
        }
    };
    
    if length >= 2 {
        let _ = texts.Item(0).map(|item| item.SetInnerText(&HSTRING::from(title)));
        let _ = texts.Item(1).map(|item| item.SetInnerText(&HSTRING::from(body)));
    }
    
    let toast = match ToastNotification::CreateToastNotification(&xml) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create toast: {}", e);
            return;
        }
    };
    
    let notifier = match ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from("BatteryMonitor")) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to create notifier: {}", e);
            return;
        }
    };
    
    let _ = notifier.Show(&toast);
}