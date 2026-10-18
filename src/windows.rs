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

use crate::Alert;

/// Returns (battery percent, plugged in).
pub fn get_battery_info() -> Result<(u8, bool), String> {
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

/// Shows notifications as balloon tips from a hidden tray icon.
pub struct Notifier {
    hwnd: HWND,
}

impl Notifier {
    pub fn new() -> Option<Self> {
        let hwnd = create_message_window()?;
        add_tray_icon(hwnd);
        Some(Notifier { hwnd })
    }

    pub fn tick(&self) {
        pump_messages();
    }

    pub fn notify(&self, title: &str, body: &str, _alert: Alert) {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
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

fn set_wide(dest: &mut [u16], text: &str) {
    let encoded: Vec<u16> = text.encode_utf16().collect();
    let len = encoded.len().min(dest.len() - 1);
    dest[..len].copy_from_slice(&encoded[..len]);
    dest[len] = 0;
}
