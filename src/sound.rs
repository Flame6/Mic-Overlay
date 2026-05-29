use std::path::{Path, PathBuf};

use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{
    PlaySoundW, SND_ASYNC, SND_FILENAME, SND_MEMORY, SND_NODEFAULT,
};

use crate::util::wide;

/// Bundled defaults — play from memory when no external file is found.
static EMBEDDED_MUTE: &[u8] = include_bytes!("../assets/mute.wav");
static EMBEDDED_UNMUTE: &[u8] = include_bytes!("../assets/unmute.wav");

/// Trim whitespace and strip surrounding `"` or `'` (common when copy-pasting paths).
fn normalize(path: &str) -> String {
    let mut s = path.trim().to_string();
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        if (bytes[0] == b'"' && bytes[s.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[s.len() - 1] == b'\'')
        {
            s = s[1..s.len() - 1].trim().to_string();
        }
    }
    s
}

/// Search bases for a relative sound path (exe dir, exe/Sounds, cwd, etc.).
fn resolve(path: &str) -> Option<PathBuf> {
    let path = normalize(path);
    if path.is_empty() {
        return None;
    }

    let direct = Path::new(&path);
    if direct.is_file() {
        return Some(direct.to_path_buf());
    }

    let file_name = direct.file_name().map(|f| f.to_owned());

    let mut bases: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            bases.push(dir.to_path_buf());
            bases.push(dir.join("Sounds"));
            bases.push(dir.join("assets"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        bases.push(cwd.clone());
        bases.push(cwd.join("Sounds"));
        bases.push(cwd.join("assets"));
    }

    for base in bases {
        let candidate = base.join(&path);
        if candidate.is_file() {
            return Some(candidate);
        }
        if let Some(ref name) = file_name {
            let by_name = base.join(name);
            if by_name.is_file() {
                return Some(by_name);
            }
        }
    }

    None
}

fn embedded_for_path(path: &str) -> Option<&'static [u8]> {
    let lower = normalize(path).to_ascii_lowercase();
    if lower.ends_with("unmute.wav") {
        Some(EMBEDDED_UNMUTE)
    } else if lower.ends_with("mute.wav") {
        Some(EMBEDDED_MUTE)
    } else {
        None
    }
}

unsafe fn play_file(resolved: &Path) -> bool {
    let wide_path = wide(&resolved.to_string_lossy());
    PlaySoundW(
        PCWSTR(wide_path.as_ptr()),
        None,
        SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
    )
    .as_bool()
}

unsafe fn play_memory(data: &'static [u8]) -> bool {
    // SND_MEMORY: first parameter is a pointer to the WAV bytes in memory.
    PlaySoundW(
        PCWSTR(data.as_ptr() as *const u16),
        None,
        SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
    )
    .as_bool()
}

/// Play a WAV file asynchronously. Tries the configured path first, then falls
/// back to the embedded default sounds for mute/unmute filenames.
pub fn play(path: &str) {
    unsafe {
        if let Some(resolved) = resolve(path) {
            if play_file(&resolved) {
                return;
            }
        }
        if let Some(data) = embedded_for_path(path) {
            let _ = play_memory(data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_quotes() {
        assert_eq!(normalize(r#""C:\Sounds\mute.wav""#), r"C:\Sounds\mute.wav");
        assert_eq!(normalize("  Sounds/mute.wav  "), "Sounds/mute.wav");
    }

    #[test]
    fn embedded_mapping() {
        assert!(embedded_for_path("Sounds/mute.wav").is_some());
        assert!(embedded_for_path("Sounds/unmute.wav").is_some());
        assert!(embedded_for_path(r#""C:\foo\unmute.wav""#).is_some());
        assert!(embedded_for_path("custom.wav").is_none());
    }
}
