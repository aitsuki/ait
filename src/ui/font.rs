#[cfg(windows)]
use crate::error::{AppError, Result};

pub fn point_size_to_logical_height(point_size: i32, dpi: i32) -> i32 {
    -((point_size * dpi) / 72)
}

#[cfg(windows)]
pub fn apply_ui_font(hwnd: windows::Win32::Foundation::HWND) -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_SETFONT};

    let font = ui_font()?;
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
    use std::sync::OnceLock;
    use windows::Win32::Graphics::Gdi::HFONT;

    static FONT: OnceLock<isize> = OnceLock::new();
    if let Some(font) = FONT.get() {
        return Ok(HFONT(*font as *mut core::ffi::c_void));
    }

    let font = create_ui_font()?;
    let _ = FONT.set(font.0 as isize);
    Ok(HFONT(
        *FONT.get().unwrap_or(&(font.0 as isize)) as *mut core::ffi::c_void
    ))
}

#[cfg(windows)]
fn create_ui_font() -> Result<windows::Win32::Graphics::Gdi::HFONT> {
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
            point_size_to_logical_height(9, dpi),
            0,
            0,
            0,
            400,
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

#[cfg(test)]
mod tests {
    use super::point_size_to_logical_height;

    #[test]
    fn point_size_to_logical_height_uses_negative_logical_height() {
        assert_eq!(point_size_to_logical_height(9, 96), -12);
        assert_eq!(point_size_to_logical_height(9, 144), -18);
    }
}
