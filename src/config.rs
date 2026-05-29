use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

use crate::util::wide;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    WhenMuted = 0,
    WhenUnmuted = 1,
    Always = 2,
    Never = 3,
}

impl DisplayMode {
    pub fn from_i32(v: i32) -> DisplayMode {
        match v {
            1 => DisplayMode::WhenUnmuted,
            2 => DisplayMode::Always,
            3 => DisplayMode::Never,
            _ => DisplayMode::WhenMuted,
        }
    }

    /// Whether the overlay should be visible for the given mute state.
    pub fn should_show(self, muted: bool) -> bool {
        match self {
            DisplayMode::WhenMuted => muted,
            DisplayMode::WhenUnmuted => !muted,
            DisplayMode::Always => true,
            DisplayMode::Never => false,
        }
    }
}

const APP_NAME: &str = "MicMuteOverlay";
const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

/// Application configuration. Field names are serialized in PascalCase so the
/// JSON layout stays byte-compatible with the original C# `config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Config {
    pub hotkey: String,
    pub overlay_text: String,
    pub mute_sound: String,
    pub unmute_sound: String,
    pub font_size: i32,
    pub fore_color: String,
    pub selected_microphone_id: String,
    pub start_with_windows: bool,
    pub outline_color: String,
    pub outline_thickness: i32,
    pub display_mode: i32,
    pub click_through_mode: bool,

    // New, optional keys (default-filled for old config files).
    #[serde(rename = "TopMostRefreshMs")]
    pub topmost_refresh_ms: u32,
    pub overlay_x: Option<i32>,
    pub overlay_y: Option<i32>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: "Ctrl+Shift+M".to_string(),
            overlay_text: "Mic Muted".to_string(),
            mute_sound: "Sounds/mute.wav".to_string(),
            unmute_sound: "Sounds/unmute.wav".to_string(),
            font_size: 20,
            fore_color: "Red".to_string(),
            selected_microphone_id: String::new(),
            start_with_windows: false,
            outline_color: "Black".to_string(),
            outline_thickness: 2,
            display_mode: 0,
            click_through_mode: false,
            topmost_refresh_ms: 2500,
            overlay_x: None,
            overlay_y: None,
        }
    }
}

impl Config {
    pub fn display_mode(&self) -> DisplayMode {
        DisplayMode::from_i32(self.display_mode)
    }

    /// Directory that contains the running executable.
    fn exe_dir() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    }

    fn appdata_dir() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join(APP_NAME))
    }

    /// Existing config path to load from, preferring the exe directory and
    /// falling back to %APPDATA%.
    fn load_path() -> Option<PathBuf> {
        if let Some(dir) = Self::exe_dir() {
            let p = dir.join("config.json");
            if p.exists() {
                return Some(p);
            }
        }
        if let Some(dir) = Self::appdata_dir() {
            let p = dir.join("config.json");
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    /// Path to write config to: exe directory when writable, else %APPDATA%.
    fn save_path() -> PathBuf {
        if let Some(dir) = Self::exe_dir() {
            // Probe writability of the exe directory.
            let probe = dir.join(".micmuteoverlay_write_test");
            if std::fs::write(&probe, b"x").is_ok() {
                let _ = std::fs::remove_file(&probe);
                return dir.join("config.json");
            }
        }
        if let Some(dir) = Self::appdata_dir() {
            let _ = std::fs::create_dir_all(&dir);
            return dir.join("config.json");
        }
        PathBuf::from("config.json")
    }

    pub fn load() -> Config {
        if let Some(path) = Self::load_path() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                match serde_json::from_str::<Config>(&text) {
                    Ok(cfg) => return cfg,
                    Err(e) => {
                        eprintln!("config parse error ({e}); using defaults");
                    }
                }
            }
        }
        Config::default()
    }

    pub fn save(&self) {
        let path = Self::save_path();
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("config save error: {e}");
                }
            }
            Err(e) => eprintln!("config serialize error: {e}"),
        }
        self.apply_windows_startup(self.start_with_windows);
    }

    /// Absolute path of the running executable, quoted for the registry.
    fn quoted_exe_path() -> Option<String> {
        std::env::current_exe()
            .ok()
            .map(|p| format!("\"{}\"", p.display()))
    }

    fn apply_windows_startup(&self, enable: bool) {
        unsafe {
            let subkey = wide(RUN_KEY);
            let mut hkey = HKEY::default();
            let open = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                0,
                KEY_WRITE,
                &mut hkey,
            );
            if open != ERROR_SUCCESS {
                return;
            }
            let name = wide(APP_NAME);
            if enable {
                if let Some(exe) = Self::quoted_exe_path() {
                    let val = wide(&exe);
                    let bytes = std::slice::from_raw_parts(
                        val.as_ptr() as *const u8,
                        val.len() * std::mem::size_of::<u16>(),
                    );
                    let _ = RegSetValueExW(hkey, PCWSTR(name.as_ptr()), 0, REG_SZ, Some(bytes));
                }
            } else {
                let _ = RegDeleteValueW(hkey, PCWSTR(name.as_ptr()));
            }
            let _ = RegCloseKey(hkey);
        }
    }

    pub fn is_set_to_start_with_windows() -> bool {
        unsafe {
            let subkey = wide(RUN_KEY);
            let mut hkey = HKEY::default();
            let open = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut hkey,
            );
            if open != ERROR_SUCCESS {
                return false;
            }
            let name = wide(APP_NAME);
            let mut data_type = REG_SZ;
            let mut size: u32 = 0;
            let q = RegQueryValueExW(
                hkey,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut data_type),
                None,
                Some(&mut size),
            );
            let _ = RegCloseKey(hkey);
            q == ERROR_SUCCESS
        }
    }
}
