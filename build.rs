fn main() {
    // Compile the Windows resource script (icon + application manifest) and link
    // it into the final executable. Only relevant for Windows targets.
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_resource::compile("app.rc", embed_resource::NONE);
    }
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
