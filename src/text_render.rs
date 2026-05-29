//! Renders outlined overlay text into a 32-bit premultiplied DIB suitable for
//! `UpdateLayeredWindow`, giving true per-pixel alpha (crisp anti-aliased text
//! with no color-key fringing).

use std::ffi::c_void;
use std::sync::Once;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Gdi::{
    CreateDIBSection, DeleteObject, HBITMAP, HDC, HGDIOBJ, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS,
};
use windows::Win32::Graphics::GdiPlus::*;

use crate::util::wide;

/// GDI+ `PixelFormat32bppPARGB` (premultiplied alpha).
const PIXEL_FORMAT_32BPP_PARGB: i32 = 0x000E_200B;
/// Padding around the measured text, matching the original layout.
const PADDING: i32 = 20;

static GDIPLUS_INIT: Once = Once::new();
static mut GDIPLUS_TOKEN: usize = 0;

pub fn ensure_started() {
    GDIPLUS_INIT.call_once(|| unsafe {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..Default::default()
        };
        let mut token: usize = 0;
        let mut output = GdiplusStartupOutput::default();
        let _ = GdiplusStartup(&mut token, &input, &mut output);
        GDIPLUS_TOKEN = token;
    });
}

/// A rendered text bitmap. Owns the DIB and frees it on drop.
pub struct RenderedText {
    pub hbitmap: HBITMAP,
    pub width: i32,
    pub height: i32,
}

impl Drop for RenderedText {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.hbitmap.0));
        }
    }
}

struct Font {
    family: *mut GpFontFamily,
    font: *mut GpFont,
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            if !self.font.is_null() {
                let _ = GdipDeleteFont(self.font);
            }
            if !self.family.is_null() {
                let _ = GdipDeleteFontFamily(self.family);
            }
        }
    }
}

unsafe fn make_font(size: f32) -> Option<Font> {
    let mut family: *mut GpFontFamily = std::ptr::null_mut();
    let name = wide("Segoe UI");
    if GdipCreateFontFamilyFromName(PCWSTR(name.as_ptr()), std::ptr::null_mut(), &mut family) != Status(0)
    {
        // Fall back to a font that always exists.
        let fallback = wide("Arial");
        if GdipCreateFontFamilyFromName(
            PCWSTR(fallback.as_ptr()),
            std::ptr::null_mut(),
            &mut family,
        ) != Status(0)
        {
            return None;
        }
    }
    let mut font: *mut GpFont = std::ptr::null_mut();
    // style 1 == FontStyleBold; UnitPoint matches WinForms point sizing.
    if GdipCreateFont(family, size, 1, UnitPoint, &mut font) != Status(0) {
        let _ = GdipDeleteFontFamily(family);
        return None;
    }
    Some(Font { family, font })
}

unsafe fn make_format() -> *mut GpStringFormat {
    let mut format: *mut GpStringFormat = std::ptr::null_mut();
    if GdipCreateStringFormat(0, 0, &mut format) != Status(0) {
        return std::ptr::null_mut();
    }
    let _ = GdipSetStringFormatAlign(format, StringAlignmentCenter);
    let _ = GdipSetStringFormatLineAlign(format, StringAlignmentCenter);
    format
}

/// Render `text` with an outline into a premultiplied DIB.
pub fn render(
    text: &str,
    font_size: i32,
    fore_argb: u32,
    outline_argb: u32,
    outline_thickness: i32,
) -> Option<RenderedText> {
    if text.is_empty() {
        return None;
    }
    ensure_started();

    unsafe {
        let font = make_font(font_size as f32)?;
        let format = make_format();
        if format.is_null() {
            return None;
        }

        let text_w = wide(text);
        let text_pcwstr = PCWSTR(text_w.as_ptr());
        let text_len = text.encode_utf16().count() as i32;

        // --- Measure on a throwaway 1x1 surface ---
        let (mut measure_bmp, mut measure_gfx): (*mut GpBitmap, *mut GpGraphics) =
            (std::ptr::null_mut(), std::ptr::null_mut());
        if GdipCreateBitmapFromScan0(1, 1, 0, PIXEL_FORMAT_32BPP_PARGB, None, &mut measure_bmp)
            != Status(0)
        {
            let _ = GdipDeleteStringFormat(format);
            return None;
        }
        GdipGetImageGraphicsContext(measure_bmp as *mut GpImage, &mut measure_gfx);
        let layout = RectF {
            X: 0.0,
            Y: 0.0,
            Width: 4096.0,
            Height: 4096.0,
        };
        let mut bounds = RectF {
            X: 0.0,
            Y: 0.0,
            Width: 0.0,
            Height: 0.0,
        };
        GdipMeasureString(
            measure_gfx,
            text_pcwstr,
            text_len,
            font.font,
            &layout,
            format,
            &mut bounds,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let _ = GdipDeleteGraphics(measure_gfx);
        let _ = GdipDisposeImage(measure_bmp as *mut GpImage);

        let outline_pad = outline_thickness.max(0) * 2;
        let width = (bounds.Width.ceil() as i32 + PADDING + outline_pad).max(1);
        let height = (bounds.Height.ceil() as i32 + PADDING + outline_pad).max(1);

        // --- Create the target DIB (top-down 32bpp) ---
        let mut bits: *mut c_void = std::ptr::null_mut();
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // negative => top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let hbitmap = match CreateDIBSection(
            HDC::default(),
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            HANDLE::default(),
            0,
        ) {
            core::result::Result::Ok(hb) if !bits.is_null() => hb,
            _ => {
                let _ = GdipDeleteStringFormat(format);
                return None;
            }
        };

        // --- Wrap the DIB memory as a GDI+ surface and draw ---
        let mut bmp: *mut GpBitmap = std::ptr::null_mut();
        if GdipCreateBitmapFromScan0(
            width,
            height,
            width * 4,
            PIXEL_FORMAT_32BPP_PARGB,
            Some(bits as *const u8),
            &mut bmp,
        ) != Status(0)
        {
            let _ = GdipDeleteStringFormat(format);
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            return None;
        }

        let mut gfx: *mut GpGraphics = std::ptr::null_mut();
        GdipGetImageGraphicsContext(bmp as *mut GpImage, &mut gfx);
        GdipSetSmoothingMode(gfx, SmoothingModeAntiAlias);
        GdipSetTextRenderingHint(gfx, TextRenderingHintAntiAlias);
        GdipGraphicsClear(gfx, 0x0000_0000);

        let full = RectF {
            X: 0.0,
            Y: 0.0,
            Width: width as f32,
            Height: height as f32,
        };

        // Outline: draw the text repeatedly with offsets.
        if outline_thickness > 0 {
            let mut outline_brush: *mut GpSolidFill = std::ptr::null_mut();
            if GdipCreateSolidFill(outline_argb, &mut outline_brush) == Status(0) {
                for dx in -outline_thickness..=outline_thickness {
                    for dy in -outline_thickness..=outline_thickness {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let rect = RectF {
                            X: dx as f32,
                            Y: dy as f32,
                            Width: width as f32,
                            Height: height as f32,
                        };
                        GdipDrawString(
                            gfx,
                            text_pcwstr,
                            text_len,
                            font.font,
                            &rect,
                            format,
                            outline_brush as *const GpBrush,
                        );
                    }
                }
                let _ = GdipDeleteBrush(outline_brush as *mut GpBrush);
            }
        }

        // Main text on top.
        let mut text_brush: *mut GpSolidFill = std::ptr::null_mut();
        if GdipCreateSolidFill(fore_argb, &mut text_brush) == Status(0) {
            GdipDrawString(
                gfx,
                text_pcwstr,
                text_len,
                font.font,
                &full,
                format,
                text_brush as *const GpBrush,
            );
            let _ = GdipDeleteBrush(text_brush as *mut GpBrush);
        }

        let _ = GdipDeleteGraphics(gfx);
        let _ = GdipDisposeImage(bmp as *mut GpImage);
        let _ = GdipDeleteStringFormat(format);

        Some(RenderedText {
            hbitmap,
            width,
            height,
        })
    }
}
