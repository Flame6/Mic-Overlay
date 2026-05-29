use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, GetDC, ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER,
    BLENDFUNCTION, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::text_render::RenderedText;
use crate::util::wide;

pub const CLASS_NAME: &str = "MicMuteOverlayWindow";

/// Register the overlay window class. Safe to call once at startup.
pub fn register_class(wnd_proc: WNDPROC) -> bool {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let class_name = wide(CLASS_NAME);
        let cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: wnd_proc,
            hInstance: HINSTANCE(hinstance.0),
            hCursor: cursor,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassW(&wc) != 0
    }
}

/// Create the always-on-top, layered, click-through-capable overlay window.
pub fn create_window() -> Option<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let hinst = HINSTANCE(hinstance.0);
        let class_name = wide(CLASS_NAME);
        let title = wide("MicMuteOverlay");
        let ex_style =
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE;
        CreateWindowExW(
            ex_style,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            hinst,
            None,
        )
        .ok()
    }
}

/// Push the rendered bitmap to the window via `UpdateLayeredWindow` (true
/// per-pixel alpha) and show it without stealing focus.
pub fn show_rendered(hwnd: HWND, rendered: &RenderedText, x: i32, y: i32) {
    unsafe {
        let screen = GetDC(None);
        let memdc = CreateCompatibleDC(screen);
        let old = SelectObject(memdc, HGDIOBJ(rendered.hbitmap.0));

        let size = SIZE {
            cx: rendered.width,
            cy: rendered.height,
        };
        let src = POINT { x: 0, y: 0 };
        let dst = POINT { x, y };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = UpdateLayeredWindow(
            hwnd,
            screen,
            Some(&dst),
            Some(&size),
            memdc,
            Some(&src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );

        SelectObject(memdc, old);
        let _ = DeleteDC(memdc);
        ReleaseDC(None, screen);

        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        reassert_topmost(hwnd);
    }
}

pub fn hide(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn is_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

/// Re-assert HWND_TOPMOST without moving, resizing, or activating the window.
pub fn reassert_topmost(hwnd: HWND) {
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

/// Toggle mouse pass-through via the layered+transparent extended styles.
pub fn apply_clickthrough(hwnd: HWND, enabled: bool) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let transparent = WS_EX_TRANSPARENT.0 as isize;
        let new_ex = if enabled {
            ex | transparent
        } else {
            ex & !transparent
        };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex);
    }
}

/// Current window rectangle as (x, y, width, height).
pub fn window_rect(hwnd: HWND) -> (i32, i32, i32, i32) {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            (
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
            )
        } else {
            (0, 0, 0, 0)
        }
    }
}

/// Begin a window move loop as if the title bar were grabbed.
pub fn begin_drag(hwnd: HWND) {
    unsafe {
        let _ = ReleaseCapture();
        let _ = SendMessageW(hwnd, WM_NCLBUTTONDOWN, WPARAM(HTCAPTION as usize), LPARAM(0));
    }
}
