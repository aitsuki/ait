#[cfg(windows)]
use crate::error::{AppError, Result};

const UI_FONT_POINT_SIZE: i32 = 11;

pub fn ui_font_point_size() -> i32 {
    UI_FONT_POINT_SIZE
}

pub fn point_size_to_logical_height(point_size: i32, dpi: i32) -> i32 {
    -((point_size * dpi) / 72)
}

#[cfg(windows)]
pub fn apply_ui_font(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    set_font(hwnd, ui_font()?)
}

#[cfg(windows)]
pub fn apply_secondary_font(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    static FONT: std::sync::OnceLock<isize> = std::sync::OnceLock::new();
    set_font(hwnd, cached_font(&FONT, 10, 400)?)
}

#[cfg(windows)]
pub fn apply_heading_font(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    static FONT: std::sync::OnceLock<isize> = std::sync::OnceLock::new();
    set_font(hwnd, cached_font(&FONT, UI_FONT_POINT_SIZE, 600)?)
}

#[cfg(windows)]
fn set_font(
    hwnd: windows::Win32::Foundation::HWND,
    font: windows::Win32::Graphics::Gdi::HFONT,
) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_SETFONT};
    unsafe {
        let _ = SendMessageW(
            hwnd,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
    Ok(())
}

#[cfg(windows)]
fn ui_font() -> Result<windows::Win32::Graphics::Gdi::HFONT> {
    static FONT: std::sync::OnceLock<isize> = std::sync::OnceLock::new();
    cached_font(&FONT, UI_FONT_POINT_SIZE, 400)
}

#[cfg(windows)]
fn cached_font(
    cache: &std::sync::OnceLock<isize>,
    point_size: i32,
    weight: i32,
) -> Result<windows::Win32::Graphics::Gdi::HFONT> {
    use windows::Win32::Graphics::Gdi::{DeleteObject, HFONT};

    if let Some(&font) = cache.get() {
        return Ok(HFONT(font as *mut core::ffi::c_void));
    }
    let font = create_ui_font(point_size, weight)?;
    if cache.set(font.0 as isize).is_err() {
        unsafe {
            let _ = DeleteObject(font.into());
        }
    }
    Ok(HFONT(*cache.get().unwrap() as *mut core::ffi::c_void))
}

#[cfg(windows)]
fn create_ui_font(point_size: i32, weight: i32) -> Result<windows::Win32::Graphics::Gdi::HFONT> {
    use windows::Win32::Graphics::Gdi::{
        CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, DEFAULT_PITCH, FF_DONTCARE, GetDC,
        GetDeviceCaps, HDC, LOGPIXELSY, OUT_DEFAULT_PRECIS, PROOF_QUALITY, ReleaseDC,
    };
    use windows::core::PCWSTR;

    let hdc = unsafe { GetDC(None) };
    let dpi = if hdc == HDC::default() {
        96
    } else {
        let dpi = unsafe { GetDeviceCaps(Some(hdc), LOGPIXELSY) };
        unsafe {
            let _ = ReleaseDC(None, hdc);
        }
        dpi.max(1)
    };
    let face = wide("Microsoft YaHei UI");
    let font = unsafe {
        CreateFontW(
            point_size_to_logical_height(point_size, dpi),
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            PROOF_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            PCWSTR(face.as_ptr()),
        )
    };

    if font == windows::Win32::Graphics::Gdi::HFONT::default() {
        Err(AppError::Windows("创建 UI 字体失败".to_string()))
    } else {
        Ok(font)
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
