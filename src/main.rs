#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod colors;
mod config;
mod hotkey;
mod hotkey_recorder;
mod keys;
mod overlay;
mod settings;
mod sound;
mod text_render;
mod tray;
mod util;

use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, SetWindowLongPtrW, TranslateMessage, GWLP_USERDATA, MSG,
};

fn main() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        text_render::ensure_started();

        overlay::register_class(Some(app::wnd_proc));
        let hwnd = match overlay::create_window() {
            Some(h) => h,
            None => return,
        };

        // The App is heap-allocated and kept alive for the whole run; a raw
        // pointer to it lives in the window's user data for the wnd procs.
        let mut app = Box::new(app::App::new());
        let app_ptr: *mut app::App = &mut *app;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_ptr as isize);
        app.attach(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Persist config on exit; hotkey + tray are released via Drop.
        app.config.save();
    }
}
