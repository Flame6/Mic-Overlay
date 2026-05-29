//! Maps the named colors offered in settings to 0xAARRGGBB values (matching
//! the .NET `Color.FromName` values the original app used). Unknown names fall
//! back to opaque white.

pub fn name_to_argb(name: &str) -> u32 {
    let rgb: u32 = match name.trim().to_ascii_lowercase().as_str() {
        "red" => 0xFF0000,
        "white" => 0xFFFFFF,
        "black" => 0x000000,
        "blue" => 0x0000FF,
        "green" => 0x008000,
        "lime" => 0x00FF00,
        "yellow" => 0xFFFF00,
        "orange" => 0xFFA500,
        "purple" => 0x800080,
        "pink" => 0xFFC0CB,
        "cyan" => 0x00FFFF,
        "magenta" => 0xFF00FF,
        "gray" | "grey" => 0x808080,
        "darkred" => 0x8B0000,
        "darkblue" => 0x00008B,
        "darkgreen" => 0x006400,
        "navy" => 0x000080,
        "maroon" => 0x800000,
        _ => 0xFFFFFF,
    };
    0xFF00_0000 | rgb
}
