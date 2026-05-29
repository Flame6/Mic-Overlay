use std::sync::Mutex;

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::keys::{modifier_priority, vk_to_name};

/// Custom message posted to the settings window when a hotkey is captured.
pub const WM_HOTKEY_RECORDED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 1;

struct RecorderInner {
    hook: isize,
    notify_hwnd: isize,
    recording: bool,
    pressed: Vec<u16>,
    all_pressed: Vec<u16>,
    result: Option<String>,
}

static RECORDER: Mutex<RecorderInner> = Mutex::new(RecorderInner {
    hook: 0,
    notify_hwnd: 0,
    recording: false,
    pressed: Vec::new(),
    all_pressed: Vec::new(),
    result: None,
});

pub fn is_recording() -> bool {
    RECORDER.lock().map(|r| r.recording).unwrap_or(false)
}

pub fn take_result() -> Option<String> {
    RECORDER.lock().ok().and_then(|mut r| r.result.take())
}

/// Begin capturing the next key combination. `notify_hwnd` receives
/// [`WM_HOTKEY_RECORDED`] once a combination is captured.
pub fn start_recording(notify_hwnd: isize) {
    stop_recording();
    unsafe {
        let hmod = GetModuleHandleW(None).unwrap_or_default();
        if let Ok(hook) =
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), HINSTANCE(hmod.0), 0)
        {
            if let Ok(mut r) = RECORDER.lock() {
                r.hook = hook.0 as isize;
                r.notify_hwnd = notify_hwnd;
                r.recording = true;
                r.pressed.clear();
                r.all_pressed.clear();
                r.result = None;
            }
        }
    }
}

pub fn stop_recording() {
    let hook = {
        let mut guard = match RECORDER.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        guard.recording = false;
        let h = guard.hook;
        guard.hook = 0;
        h
    };
    if hook != 0 {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(hook as *mut _));
        }
    }
}

fn build_string(keys: &[u16]) -> String {
    let mut names: Vec<String> = keys.iter().filter_map(|&vk| vk_to_name(vk)).collect();
    names.sort();
    names.dedup();
    names.sort_by(|a, b| {
        modifier_priority(a)
            .cmp(&modifier_priority(b))
            .then_with(|| a.cmp(b))
    });
    names.join("+")
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let msg = wparam.0 as u32;
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode as u16;
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        let mut notify: Option<isize> = None;
        if let Ok(mut r) = RECORDER.lock() {
            if r.recording && (is_down || is_up) {
                if is_down {
                    if !r.pressed.contains(&vk) {
                        r.pressed.push(vk);
                    }
                    if !r.all_pressed.contains(&vk) {
                        r.all_pressed.push(vk);
                    }
                } else {
                    r.pressed.retain(|&k| k != vk);
                    if r.pressed.is_empty() && !r.all_pressed.is_empty() {
                        let s = build_string(&r.all_pressed);
                        if !s.is_empty() {
                            r.result = Some(s);
                            r.recording = false;
                            notify = Some(r.notify_hwnd);
                        }
                    }
                }
            }
        }
        if let Some(hwnd) = notify {
            if hwnd != 0 {
                let _ = PostMessageW(
                    HWND(hwnd as *mut _),
                    WM_HOTKEY_RECORDED,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}
