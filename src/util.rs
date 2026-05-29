use windows::core::PWSTR;

/// Convert a Rust string into a NUL-terminated UTF-16 buffer.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Read a wide, NUL-terminated string starting at `ptr` into a Rust `String`.
///
/// # Safety
/// `ptr` must point to a valid NUL-terminated UTF-16 string (or be null).
pub unsafe fn from_wide_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}

/// Read a wide string from a `PWSTR`.
///
/// # Safety
/// Same requirements as [`from_wide_ptr`].
pub unsafe fn from_pwstr(p: PWSTR) -> String {
    from_wide_ptr(p.0)
}
