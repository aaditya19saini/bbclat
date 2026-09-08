#![windows_subsystem = "windows"]

use std::thread;
use std::time::Duration;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_TIP, NIIF_INFO, NIM_ADD, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, LoadIconW, PeekMessageW, RegisterClassExW,
    TranslateMessage, HICON, HWND_MESSAGE, IDI_APPLICATION, MSG, PM_REMOVE, WNDCLASSEXW,
};

const CHECK_INTERVAL_SECS: u64 = 10;
const HIGH_THRESHOLD: u8 = 95;
const LOW_THRESHOLD: u8 = 20;
const CRITICAL_LOW_THRESHOLD: u8 = 15;

fn main() {
    let Some(hwnd) = create_message_window() else {
        return;
    };
    add_tray_icon(hwnd);

    let mut last_high_notified = false;
    let mut last_low_notified = false;
    let mut last_critical_notified = false;

    loop {
        pump_messages();

        if let Ok((percent, charging)) = get_battery_info() {
            if charging && percent >= HIGH_THRESHOLD && !last_high_notified {
                notify(hwnd, &format!("Battery at {}%", percent), "Consider unplugging the charger to preserve battery health.");
                last_high_notified = true;
                last_low_notified = false;
                last_critical_notified = false;
            } else if !charging && percent <= CRITICAL_LOW_THRESHOLD && !last_critical_notified {
                notify(hwnd, &format!("Battery at {}%", percent), "plug your charger, you dumbass");
                last_critical_notified = true;
                last_low_notified = true;
                last_high_notified = false;
            } else if !charging && percent <= LOW_THRESHOLD && !last_low_notified {
                notify(hwnd, &format!("Battery at {}%", percent), "Plug in the charger.");
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

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn create_message_window() -> Option<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None).ok()?;
        let class_name = w!("bbclat_message_window");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        if RegisterClassExW(&wc) == 0 {
            return None;
        }

        CreateWindowExW(
            Default::default(),
            class_name,
            PCWSTR::null(),
            Default::default(),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            instance,
            None,
        )
        .ok()
    }
}

fn pump_messages() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn add_tray_icon(hwnd: HWND) {
    unsafe {
        let hicon: HICON = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();

        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_TIP,
            hIcon: hicon,
            ..Default::default()
        };
        set_wide(&mut nid.szTip, "bbclat battery monitor");

        let _ = Shell_NotifyIconW(NIM_ADD, &nid);
    }
}

fn notify(hwnd: HWND, title: &str, body: &str) {
    unsafe {
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_INFO,
            dwInfoFlags: NIIF_INFO,
            ..Default::default()
        };
        set_wide(&mut nid.szInfoTitle, title);
        set_wide(&mut nid.szInfo, body);

        let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

fn set_wide(dest: &mut [u16], text: &str) {
    let encoded: Vec<u16> = text.encode_utf16().collect();
    let len = encoded.len().min(dest.len() - 1);
    dest[..len].copy_from_slice(&encoded[..len]);
    dest[len] = 0;
}
