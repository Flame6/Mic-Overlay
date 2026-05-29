use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{LoadIconW, HICON};

/// Tray callback message delivered to the overlay window.
pub const WM_TRAYICON: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 2;
const TRAY_UID: u32 = 1;

/// Owns the notification-area icon; removes it on drop.
pub struct Tray {
    data: NOTIFYICONDATAW,
}

impl Tray {
    pub fn new(hwnd: HWND) -> Tray {
        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: TRAY_UID,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            ..Default::default()
        };
        data.hIcon = load_icon();

        let tip = "MicMuteOverlay";
        for (i, c) in tip.encode_utf16().enumerate() {
            if i < data.szTip.len() - 1 {
                data.szTip[i] = c;
            }
        }

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &data);
        }
        Tray { data }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.data);
        }
    }
}

fn load_icon() -> HICON {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        // Resource id 1 (IDI_APP_ICON from app.rc).
        match LoadIconW(HINSTANCE(hinstance.0), PCWSTR(1 as *const u16)) {
            Ok(icon) => icon,
            Err(_) => HICON::default(),
        }
    }
}
