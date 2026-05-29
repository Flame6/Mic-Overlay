use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN,
};

use crate::keys::{is_modifier_name, name_to_vk};

/// Fixed id for our single global hotkey registration.
pub const HOTKEY_ID: i32 = 0xB0B;

/// Registers/refreshes a single global hotkey delivered as `WM_HOTKEY` to the
/// owning window's message queue.
pub struct HotkeyManager {
    hwnd: HWND,
    registered: bool,
}

impl HotkeyManager {
    pub fn new(hwnd: HWND, hotkey: &str) -> HotkeyManager {
        let mut mgr = HotkeyManager {
            hwnd,
            registered: false,
        };
        mgr.update(hotkey);
        mgr
    }

    /// Parse a hotkey string into Win32 modifier flags and a virtual key code.
    pub fn parse(hotkey: &str) -> Option<(HOT_KEY_MODIFIERS, u16)> {
        let mut mods = HOT_KEY_MODIFIERS(0);
        let mut vk: Option<u16> = None;
        for raw in hotkey.split('+') {
            let part = raw.trim();
            if part.is_empty() {
                continue;
            }
            if is_modifier_name(part) {
                match part.to_ascii_lowercase().as_str() {
                    "ctrl" | "control" => mods |= MOD_CONTROL,
                    "alt" => mods |= MOD_ALT,
                    "shift" => mods |= MOD_SHIFT,
                    "win" => mods |= MOD_WIN,
                    _ => {}
                }
            } else if let Some(code) = name_to_vk(part) {
                vk = Some(code);
            }
        }
        vk.map(|v| (mods | MOD_NOREPEAT, v))
    }

    pub fn update(&mut self, hotkey: &str) {
        self.unregister();
        if let Some((mods, vk)) = Self::parse(hotkey) {
            unsafe {
                self.registered =
                    RegisterHotKey(self.hwnd, HOTKEY_ID, mods, vk as u32).is_ok();
            }
        }
    }

    fn unregister(&mut self) {
        if self.registered {
            unsafe {
                let _ = UnregisterHotKey(self.hwnd, HOTKEY_ID);
            }
            self.registered = false;
        }
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.unregister();
    }
}
