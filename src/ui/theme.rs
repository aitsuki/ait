#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[cfg(windows)]
    pub(crate) fn colorref(self) -> windows::Win32::Foundation::COLORREF {
        windows::Win32::Foundation::COLORREF(
            self.r as u32 | ((self.g as u32) << 8) | ((self.b as u32) << 16),
        )
    }
}

pub const COLOR_PRIMARY: RgbColor = RgbColor::new(37, 99, 235);
pub const COLOR_PRIMARY_HOVER: RgbColor = RgbColor::new(30, 90, 224);
pub const COLOR_PRIMARY_PRESSED: RgbColor = RgbColor::new(29, 78, 216);
pub const COLOR_PRIMARY_TEXT: RgbColor = RgbColor::new(255, 255, 255);
pub const COLOR_DANGER: RgbColor = RgbColor::new(220, 38, 38);
pub const COLOR_DANGER_HOVER: RgbColor = RgbColor::new(185, 28, 28);
pub const COLOR_DANGER_SOFT: RgbColor = RgbColor::new(254, 242, 242);
pub const COLOR_FOCUS_SOFT: RgbColor = RgbColor::new(219, 234, 254);
pub const COLOR_FOCUS_TEXT: RgbColor = RgbColor::new(30, 64, 175);
pub const COLOR_TEXT: RgbColor = RgbColor::new(31, 41, 55);
pub const COLOR_SURFACE: RgbColor = RgbColor::new(255, 255, 255);
pub const COLOR_SURFACE_SUBTLE: RgbColor = RgbColor::new(248, 250, 252);
pub const COLOR_SURFACE_HOVER: RgbColor = RgbColor::new(241, 245, 249);
pub const COLOR_SURFACE_PRESSED: RgbColor = RgbColor::new(226, 232, 240);
pub const COLOR_BORDER: RgbColor = RgbColor::new(203, 213, 225);
pub const COLOR_BORDER_STRONG: RgbColor = RgbColor::new(148, 163, 184);
pub const COLOR_DISABLED_SURFACE: RgbColor = RgbColor::new(243, 244, 246);
pub const COLOR_DISABLED_BORDER: RgbColor = RgbColor::new(209, 213, 219);
pub const COLOR_DISABLED_TEXT: RgbColor = RgbColor::new(156, 163, 175);

pub const CONTROL_HEIGHT: i32 = 34;
pub const PRIMARY_BUTTON_HEIGHT: i32 = 36;
pub const BUTTON_MIN_WIDTH: i32 = 72;
pub const LIST_ITEM_HEIGHT: u32 = 36;
pub const CONTROL_RADIUS: i32 = 7;
pub const FOCUS_RING_INSET: i32 = 2;
pub const SPACE_SM: i32 = 8;
pub const SPACE_MD: i32 = 12;

pub const BASE_DPI: u32 = 96;

pub fn scale_for_dpi(value: i32, dpi: u32) -> i32 {
    ((value as i64 * dpi.max(1) as i64 + BASE_DPI as i64 / 2) / BASE_DPI as i64) as i32
}

#[cfg(windows)]
pub fn system_dpi() -> u32 {
    use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, LOGPIXELSX, ReleaseDC};

    let hdc = unsafe { GetDC(None) };
    if hdc.is_invalid() {
        return BASE_DPI;
    }
    let dpi = unsafe { GetDeviceCaps(Some(hdc), LOGPIXELSX) }.max(BASE_DPI as i32) as u32;
    unsafe {
        let _ = ReleaseDC(None, hdc);
    }
    dpi
}

#[cfg(windows)]
pub fn scale(value: i32) -> i32 {
    scale_for_dpi(value, system_dpi())
}

#[cfg(test)]
mod tests {
    use super::{CONTROL_HEIGHT, LIST_ITEM_HEIGHT, scale_for_dpi};

    #[test]
    fn standard_component_metrics_are_consistent() {
        assert_eq!(CONTROL_HEIGHT, 34);
        assert_eq!(LIST_ITEM_HEIGHT, 36);
    }

    #[test]
    fn dpi_scaling_uses_96_dpi_baseline() {
        assert_eq!(scale_for_dpi(34, 96), 34);
        assert_eq!(scale_for_dpi(34, 120), 43);
        assert_eq!(scale_for_dpi(34, 144), 51);
    }
}
