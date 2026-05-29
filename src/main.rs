#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
mod hotkey;
mod hotkey_recorder;
mod keys;
mod util;

use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

fn main() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let cfg = config::Config::load();
    let controller = audio::MicController::new(&cfg.selected_microphone_id);
    if let Some(c) = &controller {
        let _ = c.available_microphones();
        println!("muted: {}", c.is_muted());
    }
    println!("loaded: {:?}", cfg);
}
