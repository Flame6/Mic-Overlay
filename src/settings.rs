use std::sync::Once;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetStockObject, GetSysColorBrush, COLOR_BTNFACE, DEFAULT_GUI_FONT, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::app::App;
use crate::audio::MicInfo;
use crate::config::Config;
use crate::hotkey_recorder::{self, WM_HOTKEY_RECORDED};
use crate::util::wide;

const CLASS_NAME: &str = "MicMuteOverlaySettings";

// Control ids.
const ID_RECORD: u32 = 101;
const ID_SAVE: u32 = 102;
const ID_CANCEL: u32 = 103;
const ID_PREVIEW: u32 = 104;

// Raw style bits not exposed as WINDOW_STYLE in the crate.
const ES_AUTOHSCROLL: u32 = 0x0080;
const ES_READONLY: u32 = 0x0800;
const CBS_DROPDOWNLIST: u32 = 0x0003;
const BS_AUTOCHECKBOX: u32 = 0x0003;
const WS_TABSTOP_BIT: u32 = 0x0001_0000;

const COLORS: [&str; 17] = [
    "Red", "White", "Black", "Blue", "Green", "Yellow", "Orange", "Purple", "Pink", "Cyan",
    "Magenta", "Gray", "DarkRed", "DarkBlue", "DarkGreen", "Navy", "Maroon",
];
const MODES: [&str; 4] = ["When Muted", "When Unmuted", "Always", "Never"];

struct Settings {
    app: *mut App,
    hotkey_edit: HWND,
    text_edit: HWND,
    mute_edit: HWND,
    unmute_edit: HWND,
    font_edit: HWND,
    thickness_edit: HWND,
    color_combo: HWND,
    outline_combo: HWND,
    mic_combo: HWND,
    mode_combo: HWND,
    start_check: HWND,
    click_check: HWND,
    record_btn: HWND,
    mics: Vec<MicInfo>,
}

static REGISTER: Once = Once::new();

fn register_class() {
    REGISTER.call_once(|| unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let class_name = wide(CLASS_NAME);
        let cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let wc = WNDCLASSW {
            lpfnWndProc: Some(settings_proc),
            hInstance: windows::Win32::Foundation::HINSTANCE(hinstance.0),
            hCursor: cursor,
            hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassW(&wc);
    });
}

/// Open (or focus) the settings window for the given app.
pub fn open(app: *mut App) {
    if app.is_null() {
        return;
    }
    unsafe {
        let existing = (*app).settings_hwnd;
        if existing.0 as usize != 0 && IsWindow(existing).as_bool() {
            let _ = SetForegroundWindow(existing);
            return;
        }
        register_class();
        create_window(app);
    }
}

unsafe fn font() -> HGDIOBJ {
    GetStockObject(DEFAULT_GUI_FONT)
}

unsafe fn child(
    parent: HWND,
    class: &str,
    text: &str,
    style_extra: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: u32,
) -> HWND {
    let hinstance = GetModuleHandleW(None).unwrap_or_default();
    let cls = wide(class);
    let txt = wide(text);
    let style = WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | style_extra);
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        PCWSTR(cls.as_ptr()),
        PCWSTR(txt.as_ptr()),
        style,
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as usize as *mut _),
        windows::Win32::Foundation::HINSTANCE(hinstance.0),
        None,
    )
    .unwrap_or_default();
    let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font().0 as usize), LPARAM(1));
    hwnd
}

unsafe fn label(parent: HWND, text: &str, x: i32, y: i32, w: i32, h: i32) -> HWND {
    child(parent, "STATIC", text, 0, x, y, w, h, 0)
}

unsafe fn edit(parent: HWND, text: &str, x: i32, y: i32, w: i32, h: i32, id: u32, readonly: bool) -> HWND {
    let mut extra = WS_BORDER.0 | WS_TABSTOP_BIT | ES_AUTOHSCROLL;
    if readonly {
        extra |= ES_READONLY;
    }
    child(parent, "EDIT", text, extra, x, y, w, h, id)
}

unsafe fn button(parent: HWND, text: &str, x: i32, y: i32, w: i32, h: i32, id: u32) -> HWND {
    child(parent, "BUTTON", text, WS_TABSTOP_BIT, x, y, w, h, id)
}

unsafe fn checkbox(parent: HWND, text: &str, x: i32, y: i32, w: i32, h: i32, id: u32) -> HWND {
    child(parent, "BUTTON", text, BS_AUTOCHECKBOX | WS_TABSTOP_BIT, x, y, w, h, id)
}

unsafe fn combo(parent: HWND, x: i32, y: i32, w: i32, h: i32, id: u32) -> HWND {
    let extra = CBS_DROPDOWNLIST | WS_TABSTOP_BIT | WS_VSCROLL.0;
    // Make the drop-down list tall enough to be usable.
    child(parent, "COMBOBOX", "", extra, x, y, w, h + 200, id)
}

unsafe fn combo_add(combo: HWND, text: &str) {
    let w = wide(text);
    let _ = SendMessageW(combo, CB_ADDSTRING, WPARAM(0), LPARAM(w.as_ptr() as isize));
}

unsafe fn combo_set(combo: HWND, index: i32) {
    let _ = SendMessageW(combo, CB_SETCURSEL, WPARAM(index.max(0) as usize), LPARAM(0));
}

unsafe fn combo_get(combo: HWND) -> i32 {
    SendMessageW(combo, CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as i32
}

unsafe fn check_set(hwnd: HWND, checked: bool) {
    let _ = SendMessageW(hwnd, BM_SETCHECK, WPARAM(if checked { 1 } else { 0 }), LPARAM(0));
}

unsafe fn check_get(hwnd: HWND) -> bool {
    SendMessageW(hwnd, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == 1
}

unsafe fn get_text(hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len as usize + 1];
    let n = GetWindowTextW(hwnd, &mut buf);
    String::from_utf16_lossy(&buf[..n as usize])
}

unsafe fn set_text(hwnd: HWND, text: &str) {
    let w = wide(text);
    let _ = SetWindowTextW(hwnd, PCWSTR(w.as_ptr()));
}

fn color_index(name: &str) -> i32 {
    COLORS
        .iter()
        .position(|c| c.eq_ignore_ascii_case(name.trim()))
        .map(|i| i as i32)
        .unwrap_or(0)
}

unsafe fn create_window(app: *mut App) {
    let hinstance = GetModuleHandleW(None).unwrap_or_default();
    let class_name = wide(CLASS_NAME);
    let title = wide("MicMuteOverlay Settings");
    let style = WINDOW_STYLE(WS_CAPTION.0 | WS_SYSMENU.0);

    let width = 480;
    let height = 620;

    let hwnd = match CreateWindowExW(
        WINDOW_EX_STYLE(0),
        PCWSTR(class_name.as_ptr()),
        PCWSTR(title.as_ptr()),
        style,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        width,
        height,
        None,
        None,
        windows::Win32::Foundation::HINSTANCE(hinstance.0),
        None,
    ) {
        Ok(h) => h,
        Err(_) => return,
    };

    let config: Config = (*app).config.clone();

    let label_x = 20;
    let control_x = 160;
    let control_w = 280;
    let row = 35;
    let mut y = 20;

    // Hotkey
    label(hwnd, "Hotkey:", label_x, y + 3, 120, 23);
    let hotkey_edit = edit(hwnd, &config.hotkey, control_x, y, 180, 23, 0, true);
    let record_btn = button(hwnd, "Record", control_x + 190, y, 70, 23, ID_RECORD);
    y += row;

    label(hwnd, "Overlay Text:", label_x, y + 3, 120, 23);
    let text_edit = edit(hwnd, &config.overlay_text, control_x, y, control_w, 23, 0, false);
    y += row;

    label(hwnd, "Mute Sound:", label_x, y + 3, 120, 23);
    let mute_edit = edit(hwnd, &config.mute_sound, control_x, y, control_w, 23, 0, false);
    y += row;

    label(hwnd, "Unmute Sound:", label_x, y + 3, 120, 23);
    let unmute_edit = edit(hwnd, &config.unmute_sound, control_x, y, control_w, 23, 0, false);
    y += row;

    label(hwnd, "Font Size:", label_x, y + 3, 120, 23);
    let font_edit = edit(hwnd, &config.font_size.to_string(), control_x, y, 80, 23, 0, false);
    y += row;

    label(hwnd, "Text Color:", label_x, y + 3, 120, 23);
    let color_combo = combo(hwnd, control_x, y, 120, 23, 0);
    for c in COLORS {
        combo_add(color_combo, c);
    }
    combo_set(color_combo, color_index(&config.fore_color));
    y += row;

    label(hwnd, "Outline Color:", label_x, y + 3, 120, 23);
    let outline_combo = combo(hwnd, control_x, y, 120, 23, 0);
    for c in COLORS {
        combo_add(outline_combo, c);
    }
    combo_set(outline_combo, color_index(&config.outline_color));
    y += row;

    label(hwnd, "Outline Thickness:", label_x, y + 3, 130, 23);
    let thickness_edit = edit(
        hwnd,
        &config.outline_thickness.to_string(),
        control_x,
        y,
        80,
        23,
        0,
        false,
    );
    y += row;

    // Microphone
    label(hwnd, "Microphone:", label_x, y + 3, 120, 23);
    let mic_combo = combo(hwnd, control_x, y, control_w, 23, 0);
    let mics = (*app)
        .controller
        .as_ref()
        .map(|c| c.available_microphones())
        .unwrap_or_default();
    let mut sel_mic = 0i32;
    for (i, m) in mics.iter().enumerate() {
        combo_add(mic_combo, &m.display_name());
        if m.id == config.selected_microphone_id {
            sel_mic = i as i32;
        }
    }
    combo_set(mic_combo, sel_mic);
    y += row;

    // Show overlay mode
    label(hwnd, "Show Overlay:", label_x, y + 3, 120, 23);
    let mode_combo = combo(hwnd, control_x, y, 150, 23, 0);
    for m in MODES {
        combo_add(mode_combo, m);
    }
    combo_set(mode_combo, config.display_mode);
    y += row;

    let start_check = checkbox(hwnd, "Start with Windows", label_x, y, 200, 25, 0);
    check_set(start_check, Config::is_set_to_start_with_windows());
    y += 30;

    let click_check = checkbox(
        hwnd,
        "Click-through mode (overlay ignores mouse clicks)",
        label_x,
        y,
        400,
        25,
        0,
    );
    check_set(click_check, config.click_through_mode);
    y += 45;

    button(hwnd, "Save && Close", label_x, y, 110, 30, ID_SAVE);
    button(hwnd, "Cancel", label_x + 120, y, 100, 30, ID_CANCEL);
    button(hwnd, "Preview", label_x + 230, y, 100, 30, ID_PREVIEW);

    let state = Box::new(Settings {
        app,
        hotkey_edit,
        text_edit,
        mute_edit,
        unmute_edit,
        font_edit,
        thickness_edit,
        color_combo,
        outline_combo,
        mic_combo,
        mode_combo,
        start_check,
        click_check,
        record_btn,
        mics,
    });
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

    (*app).settings_hwnd = hwnd;
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = SetForegroundWindow(hwnd);
}

unsafe fn parse_clamped(s: &str, min: i32, max: i32, default: i32) -> i32 {
    s.trim().parse::<i32>().unwrap_or(default).clamp(min, max)
}

unsafe fn do_save(s: &Settings) {
    let app = s.app;
    if app.is_null() {
        return;
    }
    (*app).cancel_preview();

    let cfg = &mut (*app).config;
    cfg.hotkey = get_text(s.hotkey_edit);
    cfg.overlay_text = get_text(s.text_edit);
    cfg.mute_sound = get_text(s.mute_edit);
    cfg.unmute_sound = get_text(s.unmute_edit);
    cfg.font_size = parse_clamped(&get_text(s.font_edit), 8, 72, 20);
    cfg.outline_thickness = parse_clamped(&get_text(s.thickness_edit), 0, 10, 2);

    let ci = combo_get(s.color_combo);
    if ci >= 0 && (ci as usize) < COLORS.len() {
        cfg.fore_color = COLORS[ci as usize].to_string();
    }
    let oi = combo_get(s.outline_combo);
    if oi >= 0 && (oi as usize) < COLORS.len() {
        cfg.outline_color = COLORS[oi as usize].to_string();
    }
    let mode = combo_get(s.mode_combo);
    if mode >= 0 {
        cfg.display_mode = mode;
    }
    cfg.start_with_windows = check_get(s.start_check);
    cfg.click_through_mode = check_get(s.click_check);

    let mic_idx = combo_get(s.mic_combo);
    if mic_idx >= 0 && (mic_idx as usize) < s.mics.len() {
        let id = s.mics[mic_idx as usize].id.clone();
        cfg.selected_microphone_id = id.clone();
        if let Some(controller) = &mut (*app).controller {
            controller.set_microphone(&id);
        }
        // Re-read mute state from the newly selected device.
        if let Some(controller) = &(*app).controller {
            (*app).muted = controller.is_muted();
        }
    }

    (*app).config.save();
    (*app).reload();
}

unsafe fn do_preview(s: &Settings) {
    let app = s.app;
    if app.is_null() {
        return;
    }
    if (*app).preview_backup.is_none() {
        (*app).preview_backup = Some((*app).config.clone());
    }
    let cfg = &mut (*app).config;
    cfg.overlay_text = get_text(s.text_edit);
    cfg.font_size = parse_clamped(&get_text(s.font_edit), 8, 72, 20);
    cfg.outline_thickness = parse_clamped(&get_text(s.thickness_edit), 0, 10, 2);
    let ci = combo_get(s.color_combo);
    if ci >= 0 && (ci as usize) < COLORS.len() {
        cfg.fore_color = COLORS[ci as usize].to_string();
    }
    let oi = combo_get(s.outline_combo);
    if oi >= 0 && (oi as usize) < COLORS.len() {
        cfg.outline_color = COLORS[oi as usize].to_string();
    }
    (*app).preview_show();
}

unsafe fn toggle_record(s: &Settings) {
    if hotkey_recorder::is_recording() {
        hotkey_recorder::stop_recording();
        set_text(s.record_btn, "Record");
    } else {
        let notify = s_window(s.record_btn);
        hotkey_recorder::start_recording(notify);
        set_text(s.record_btn, "Stop");
        set_text(s.hotkey_edit, "Press key combination...");
    }
}

/// The settings window handle (parent of the record button).
unsafe fn s_window(child: HWND) -> isize {
    GetParent(child).map(|h| h.0 as isize).unwrap_or(0)
}

unsafe extern "system" fn settings_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u32;
            let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Settings;
            if !state.is_null() {
                match id {
                    ID_RECORD => toggle_record(&*state),
                    ID_PREVIEW => do_preview(&*state),
                    ID_SAVE => {
                        do_save(&*state);
                        let _ = DestroyWindow(hwnd);
                    }
                    ID_CANCEL => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }
        m if m == WM_HOTKEY_RECORDED => {
            let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Settings;
            if !state.is_null() {
                if let Some(result) = hotkey_recorder::take_result() {
                    set_text((*state).hotkey_edit, &result);
                    set_text((*state).record_btn, "Record");
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            hotkey_recorder::stop_recording();
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Settings;
            if !ptr.is_null() {
                let state = Box::from_raw(ptr);
                if !state.app.is_null() {
                    (*state.app).settings_hwnd = HWND::default();
                    (*state.app).cancel_preview();
                }
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
