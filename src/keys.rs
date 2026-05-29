//! Bidirectional mapping between human-readable key names (as used in the
//! `config.json` hotkey string and the settings recorder) and Win32 virtual
//! key codes. Keeping both directions in one place guarantees that a hotkey
//! recorded in the UI round-trips through parsing without surprises.

use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Modifier names recognised in a hotkey string (case-insensitive).
pub fn is_modifier_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "ctrl" | "control" | "alt" | "shift" | "win"
    )
}

/// Map a key name to a Win32 virtual key code (the non-modifier key).
pub fn name_to_vk(name: &str) -> Option<u16> {
    let n = name.trim();
    // Single letters A-Z.
    if n.len() == 1 {
        let c = n.chars().next().unwrap().to_ascii_uppercase();
        if c.is_ascii_alphabetic() {
            return Some(c as u16);
        }
        if c.is_ascii_digit() {
            return Some(c as u16);
        }
    }
    let vk = match n.to_ascii_lowercase().as_str() {
        "space" => VK_SPACE,
        "enter" | "return" => VK_RETURN,
        "tab" => VK_TAB,
        "esc" | "escape" => VK_ESCAPE,
        "backspace" | "back" => VK_BACK,
        "delete" | "del" => VK_DELETE,
        "insert" | "ins" => VK_INSERT,
        "home" => VK_HOME,
        "end" => VK_END,
        "pageup" | "pgup" => VK_PRIOR,
        "pagedown" | "pgdn" => VK_NEXT,
        "up" => VK_UP,
        "down" => VK_DOWN,
        "left" => VK_LEFT,
        "right" => VK_RIGHT,
        "f1" => VK_F1,
        "f2" => VK_F2,
        "f3" => VK_F3,
        "f4" => VK_F4,
        "f5" => VK_F5,
        "f6" => VK_F6,
        "f7" => VK_F7,
        "f8" => VK_F8,
        "f9" => VK_F9,
        "f10" => VK_F10,
        "f11" => VK_F11,
        "f12" => VK_F12,
        "num0" => VK_NUMPAD0,
        "num1" => VK_NUMPAD1,
        "num2" => VK_NUMPAD2,
        "num3" => VK_NUMPAD3,
        "num4" => VK_NUMPAD4,
        "num5" => VK_NUMPAD5,
        "num6" => VK_NUMPAD6,
        "num7" => VK_NUMPAD7,
        "num8" => VK_NUMPAD8,
        "num9" => VK_NUMPAD9,
        "num*" => VK_MULTIPLY,
        "num+" => VK_ADD,
        "num-" => VK_SUBTRACT,
        "num/" => VK_DIVIDE,
        "num." => VK_DECIMAL,
        _ => return None,
    };
    Some(vk.0)
}

/// Sort priority so a built hotkey string reads "Ctrl+Alt+Shift+Win+Key".
pub fn modifier_priority(name: &str) -> u8 {
    match name {
        "Ctrl" => 1,
        "Alt" => 2,
        "Shift" => 3,
        "Win" => 4,
        _ => 5,
    }
}

/// Map a virtual key code back to a display name. Returns `None` for keys that
/// should not appear in a hotkey (e.g. unrecognised keys).
pub fn vk_to_name(vk: u16) -> Option<String> {
    let key = VIRTUAL_KEY(vk);
    let named = match key {
        VK_LCONTROL | VK_RCONTROL | VK_CONTROL => "Ctrl",
        VK_LMENU | VK_RMENU | VK_MENU => "Alt",
        VK_LSHIFT | VK_RSHIFT | VK_SHIFT => "Shift",
        VK_LWIN | VK_RWIN => "Win",
        VK_SPACE => "Space",
        VK_RETURN => "Enter",
        VK_TAB => "Tab",
        VK_ESCAPE => "Esc",
        VK_BACK => "Backspace",
        VK_DELETE => "Delete",
        VK_INSERT => "Insert",
        VK_HOME => "Home",
        VK_END => "End",
        VK_PRIOR => "PageUp",
        VK_NEXT => "PageDown",
        VK_UP => "Up",
        VK_DOWN => "Down",
        VK_LEFT => "Left",
        VK_RIGHT => "Right",
        VK_F1 => "F1",
        VK_F2 => "F2",
        VK_F3 => "F3",
        VK_F4 => "F4",
        VK_F5 => "F5",
        VK_F6 => "F6",
        VK_F7 => "F7",
        VK_F8 => "F8",
        VK_F9 => "F9",
        VK_F10 => "F10",
        VK_F11 => "F11",
        VK_F12 => "F12",
        VK_NUMPAD0 => "Num0",
        VK_NUMPAD1 => "Num1",
        VK_NUMPAD2 => "Num2",
        VK_NUMPAD3 => "Num3",
        VK_NUMPAD4 => "Num4",
        VK_NUMPAD5 => "Num5",
        VK_NUMPAD6 => "Num6",
        VK_NUMPAD7 => "Num7",
        VK_NUMPAD8 => "Num8",
        VK_NUMPAD9 => "Num9",
        VK_MULTIPLY => "Num*",
        VK_ADD => "Num+",
        VK_SUBTRACT => "Num-",
        VK_DIVIDE => "Num/",
        VK_DECIMAL => "Num.",
        _ => "",
    };
    if !named.is_empty() {
        return Some(named.to_string());
    }
    // Letters A-Z and digits 0-9.
    if (0x41..=0x5A).contains(&vk) {
        return Some((vk as u8 as char).to_string());
    }
    if (0x30..=0x39).contains(&vk) {
        return Some((vk as u8 as char).to_string());
    }
    None
}
