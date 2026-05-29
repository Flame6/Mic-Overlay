use std::ffi::c_void;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::audio::MicController;
use crate::config::Config;
use crate::hotkey::HotkeyManager;
use crate::overlay;
use crate::sound;
use crate::text_render::{self, RenderedText};
use crate::tray::{Tray, WM_TRAYICON};
use crate::util::wide;

pub const TIMER_TOPMOST: usize = 1;
pub const TIMER_PREVIEW: usize = 2;

pub const ID_SETTINGS: u32 = 1;
pub const ID_RESET: u32 = 2;
pub const ID_EXIT: u32 = 3;

/// Central application state, shared with the settings window via a raw pointer
/// stored in the overlay window's `GWLP_USERDATA`.
pub struct App {
    pub config: Config,
    pub controller: Option<MicController>,
    pub hotkey: Option<HotkeyManager>,
    pub tray: Option<Tray>,
    pub hwnd: HWND,
    pub muted: bool,
    pub x: i32,
    pub y: i32,
    pub positioned: bool,
    pub settings_hwnd: HWND,
    pub preview_backup: Option<Config>,
    pub force_preview: bool,
}

impl App {
    pub fn new() -> App {
        let config = Config::load();
        let controller = MicController::new(&config.selected_microphone_id);
        let muted = controller.as_ref().map(|c| c.is_muted()).unwrap_or(false);
        let positioned = config.overlay_x.is_some() && config.overlay_y.is_some();
        App {
            x: config.overlay_x.unwrap_or(0),
            y: config.overlay_y.unwrap_or(0),
            positioned,
            config,
            controller,
            hotkey: None,
            tray: None,
            hwnd: HWND::default(),
            muted,
            settings_hwnd: HWND::default(),
            preview_backup: None,
            force_preview: false,
        }
    }

    /// Wire the app to its window and show the initial overlay state.
    pub fn attach(&mut self, hwnd: HWND) {
        self.hwnd = hwnd;
        self.hotkey = Some(HotkeyManager::new(hwnd, &self.config.hotkey));
        self.tray = Some(Tray::new(hwnd));
        overlay::apply_clickthrough(hwnd, self.config.click_through_mode);
        unsafe {
            let interval = self.config.topmost_refresh_ms.max(250);
            SetTimer(hwnd, TIMER_TOPMOST, interval, None);
        }
        self.update_overlay(false);
    }

    fn render_current(&self) -> Option<RenderedText> {
        text_render::render(
            &self.config.overlay_text,
            self.config.font_size,
            crate::colors::name_to_argb(&self.config.fore_color),
            crate::colors::name_to_argb(&self.config.outline_color),
            self.config.outline_thickness,
        )
    }

    /// Compute a centered top-left for a window of the given size, using the
    /// primary monitor's work area.
    fn centered(&self, w: i32, h: i32) -> (i32, i32) {
        unsafe {
            let mut rect = RECT::default();
            let ok = SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                Some(&mut rect as *mut _ as *mut c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
            .is_ok();
            if ok {
                let x = rect.left + (rect.right - rect.left - w) / 2;
                let y = rect.top + (rect.bottom - rect.top - h) / 2;
                (x.max(0), y.max(0))
            } else {
                (100, 100)
            }
        }
    }

    /// Re-evaluate visibility for the current mute state and refresh the window.
    /// Plays a transition sound only when `play_sound` is true.
    pub fn update_overlay(&mut self, play_sound: bool) {
        if self.force_preview {
            return;
        }
        let show = self.config.display_mode().should_show(self.muted);
        if show {
            if let Some(rendered) = self.render_current() {
                if !self.positioned {
                    let (cx, cy) = self.centered(rendered.width, rendered.height);
                    self.x = cx;
                    self.y = cy;
                    self.positioned = true;
                }
                overlay::show_rendered(self.hwnd, &rendered, self.x, self.y);
            }
        } else {
            overlay::hide(self.hwnd);
        }

        if play_sound {
            if self.muted {
                sound::play(&self.config.mute_sound);
            } else {
                sound::play(&self.config.unmute_sound);
            }
        }
    }

    pub fn on_hotkey(&mut self) {
        if let Some(controller) = &self.controller {
            self.muted = controller.toggle_mute();
        } else {
            self.muted = !self.muted;
        }
        self.update_overlay(true);
    }

    pub fn on_topmost_timer(&self) {
        if overlay::is_visible(self.hwnd) {
            overlay::reassert_topmost(self.hwnd);
        }
    }

    /// Apply a freshly-edited config: re-register hotkey, click-through, redraw.
    pub fn reload(&mut self) {
        if let Some(hotkey) = &mut self.hotkey {
            hotkey.update(&self.config.hotkey);
        }
        overlay::apply_clickthrough(self.hwnd, self.config.click_through_mode);
        self.update_overlay(false);
    }

    pub fn reset_position(&mut self) {
        if let Some(rendered) = self.render_current() {
            let (cx, cy) = self.centered(rendered.width, rendered.height);
            self.x = cx;
            self.y = cy;
            self.positioned = true;
            self.config.overlay_x = Some(cx);
            self.config.overlay_y = Some(cy);
            self.config.save();
            overlay::show_rendered(self.hwnd, &rendered, self.x, self.y);
        }
    }

    pub fn save_position_from_window(&mut self) {
        let (x, y, _, _) = overlay::window_rect(self.hwnd);
        self.x = x;
        self.y = y;
        self.positioned = true;
        self.config.overlay_x = Some(x);
        self.config.overlay_y = Some(y);
        self.config.save();
    }

    /// Show a temporary muted preview using the (already applied) config.
    pub fn preview_show(&mut self) {
        self.force_preview = true;
        if let Some(rendered) = self.render_current() {
            if !self.positioned {
                let (cx, cy) = self.centered(rendered.width, rendered.height);
                self.x = cx;
                self.y = cy;
                self.positioned = true;
            }
            overlay::show_rendered(self.hwnd, &rendered, self.x, self.y);
        }
        unsafe {
            SetTimer(self.hwnd, TIMER_PREVIEW, 3000, None);
        }
    }

    fn end_preview(&mut self) {
        unsafe {
            let _ = KillTimer(self.hwnd, TIMER_PREVIEW);
        }
        self.force_preview = false;
        if let Some(cfg) = self.preview_backup.take() {
            self.config = cfg;
        }
        self.update_overlay(false);
    }

    /// Abort an in-progress preview without restoring the backup (used when the
    /// edited values are being committed anyway).
    pub fn cancel_preview(&mut self) {
        unsafe {
            let _ = KillTimer(self.hwnd, TIMER_PREVIEW);
        }
        self.force_preview = false;
        self.preview_backup = None;
    }
}

unsafe fn app_ptr(hwnd: HWND) -> *mut App {
    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App
}

/// Build and display a popup menu; returns the chosen command id (0 if none).
unsafe fn show_menu(hwnd: HWND, include_reset: bool) -> u32 {
    let menu = match CreatePopupMenu() {
        Ok(m) => m,
        Err(_) => return 0,
    };
    let settings = wide("Settings");
    let _ = AppendMenuW(menu, MF_STRING, ID_SETTINGS as usize, PCWSTR(settings.as_ptr()));
    if include_reset {
        let reset = wide("Reset Overlay Position");
        let _ = AppendMenuW(menu, MF_STRING, ID_RESET as usize, PCWSTR(reset.as_ptr()));
    }
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let exit = wide("Exit");
    let _ = AppendMenuW(menu, MF_STRING, ID_EXIT as usize, PCWSTR(exit.as_ptr()));

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    // Required so the menu dismisses correctly when clicking elsewhere.
    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_RIGHTBUTTON,
        pt.x,
        pt.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);
    cmd.0 as u32
}

unsafe fn handle_command(app: *mut App, hwnd: HWND, cmd: u32) {
    match cmd {
        ID_SETTINGS => crate::settings::open(app),
        ID_RESET => {
            if !app.is_null() {
                (*app).reset_position();
            }
        }
        ID_EXIT => {
            let _ = DestroyWindow(hwnd);
        }
        _ => {}
    }
}

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let app = app_ptr(hwnd);
            if !app.is_null() {
                (*app).on_hotkey();
            }
            LRESULT(0)
        }
        WM_TIMER => {
            let app = app_ptr(hwnd);
            if !app.is_null() {
                match wparam.0 {
                    TIMER_TOPMOST => (*app).on_topmost_timer(),
                    TIMER_PREVIEW => (*app).end_preview(),
                    _ => {}
                }
            }
            LRESULT(0)
        }
        WM_TRAYICON => {
            let mouse = lparam.0 as u32;
            if mouse == WM_RBUTTONUP || mouse == WM_CONTEXTMENU {
                let cmd = show_menu(hwnd, true);
                if cmd != 0 {
                    handle_command(app_ptr(hwnd), hwnd, cmd);
                }
            } else if mouse == WM_LBUTTONDBLCLK {
                crate::settings::open(app_ptr(hwnd));
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let app = app_ptr(hwnd);
            let clickthrough = !app.is_null() && (*app).config.click_through_mode;
            if !clickthrough {
                overlay::begin_drag(hwnd);
            }
            LRESULT(0)
        }
        WM_EXITSIZEMOVE => {
            let app = app_ptr(hwnd);
            if !app.is_null() {
                (*app).save_position_from_window();
            }
            LRESULT(0)
        }
        WM_RBUTTONUP | WM_CONTEXTMENU => {
            let cmd = show_menu(hwnd, false);
            if cmd != 0 {
                handle_command(app_ptr(hwnd), hwnd, cmd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
