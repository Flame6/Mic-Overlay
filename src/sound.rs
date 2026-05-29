use std::path::{Path, PathBuf};

use windows::Win32::Media::Audio::{
    PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT,
};

use crate::util::wide;

/// Resolve a (possibly relative) sound path against the common base locations,
/// mirroring the original multi-path lookup.
fn resolve(path: &str) -> Option<PathBuf> {
    if path.trim().is_empty() {
        return None;
    }
    let direct = Path::new(path);
    if direct.is_file() {
        return Some(direct.to_path_buf());
    }
    let mut bases: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            bases.push(dir.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        bases.push(cwd);
    }
    for base in bases {
        let candidate = base.join(path);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Play a WAV file asynchronously. Silently does nothing if the file cannot be
/// found so a missing sound never interrupts the app.
pub fn play(path: &str) {
    if let Some(resolved) = resolve(path) {
        let wide_path = wide(&resolved.to_string_lossy());
        unsafe {
            let _ = PlaySoundW(
                windows::core::PCWSTR(wide_path.as_ptr()),
                None,
                SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
            );
        }
    }
}
