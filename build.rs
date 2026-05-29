use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_resource::compile("app.rc", embed_resource::NONE);
        copy_sounds_to_output();
    }
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/mute.wav");
    println!("cargo:rerun-if-changed=assets/unmute.wav");
}

/// Copy bundled WAVs next to the built exe so `Sounds/mute.wav` works when
/// running directly from `target\release\` during development.
fn copy_sounds_to_output() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let mut out_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
        manifest_dir.join("target").to_string_lossy().into_owned()
    }));
    if let Ok(host) = env::var("CARGO_CFG_TARGET") {
        if host != env::var("HOST").unwrap_or_default() {
            out_dir.push(host);
        }
    }
    out_dir.push(profile);
    let sounds_dir = out_dir.join("Sounds");
    let _ = fs::create_dir_all(&sounds_dir);
    for name in ["mute.wav", "unmute.wav"] {
        let src = manifest_dir.join("assets").join(name);
        let dst = sounds_dir.join(name);
        if src.is_file() {
            let _ = fs::copy(&src, &dst);
        }
    }
}
