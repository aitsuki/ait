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

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub primary: RgbColor,
    pub primary_hover: RgbColor,
    pub primary_pressed: RgbColor,
    pub primary_text: RgbColor,
    pub danger: RgbColor,
    pub danger_hover: RgbColor,
    pub danger_soft: RgbColor,
    pub focus_soft: RgbColor,
    pub focus_text: RgbColor,
    pub text: RgbColor,
    pub text_secondary: RgbColor,
    pub placeholder: RgbColor,
    pub surface: RgbColor,
    pub surface_subtle: RgbColor,
    pub surface_hover: RgbColor,
    pub surface_pressed: RgbColor,
    pub border: RgbColor,
    pub border_strong: RgbColor,
    pub disabled_surface: RgbColor,
    pub disabled_border: RgbColor,
    pub disabled_text: RgbColor,
}

const LIGHT: ThemePalette = ThemePalette {
    primary: RgbColor::new(37, 99, 235),
    primary_hover: RgbColor::new(30, 90, 224),
    primary_pressed: RgbColor::new(29, 78, 216),
    primary_text: RgbColor::new(255, 255, 255),
    danger: RgbColor::new(220, 38, 38),
    danger_hover: RgbColor::new(185, 28, 28),
    danger_soft: RgbColor::new(254, 242, 242),
    focus_soft: RgbColor::new(219, 234, 254),
    focus_text: RgbColor::new(30, 64, 175),
    text: RgbColor::new(31, 41, 55),
    text_secondary: RgbColor::new(107, 114, 128),
    placeholder: RgbColor::new(148, 163, 184),
    surface: RgbColor::new(255, 255, 255),
    surface_subtle: RgbColor::new(248, 250, 252),
    surface_hover: RgbColor::new(241, 245, 249),
    surface_pressed: RgbColor::new(226, 232, 240),
    border: RgbColor::new(203, 213, 225),
    border_strong: RgbColor::new(148, 163, 184),
    disabled_surface: RgbColor::new(243, 244, 246),
    disabled_border: RgbColor::new(209, 213, 219),
    disabled_text: RgbColor::new(156, 163, 175),
};

const DARK: ThemePalette = ThemePalette {
    // Muted steel blue keeps dark-mode accents distinct without a neon effect.
    primary: RgbColor::new(78, 104, 140),
    primary_hover: RgbColor::new(86, 114, 149),
    primary_pressed: RgbColor::new(65, 88, 120),
    primary_text: RgbColor::new(255, 255, 255),
    danger: RgbColor::new(248, 113, 113),
    danger_hover: RgbColor::new(252, 165, 165),
    danger_soft: RgbColor::new(69, 30, 36),
    focus_soft: RgbColor::new(43, 55, 73),
    focus_text: RgbColor::new(205, 215, 229),
    text: RgbColor::new(229, 231, 235),
    text_secondary: RgbColor::new(156, 163, 175),
    placeholder: RgbColor::new(107, 114, 128),
    surface: RgbColor::new(24, 28, 36),
    surface_subtle: RgbColor::new(30, 35, 44),
    surface_hover: RgbColor::new(39, 46, 58),
    surface_pressed: RgbColor::new(51, 61, 76),
    border: RgbColor::new(65, 75, 91),
    border_strong: RgbColor::new(107, 114, 128),
    disabled_surface: RgbColor::new(32, 36, 44),
    disabled_border: RgbColor::new(48, 55, 66),
    disabled_text: RgbColor::new(107, 114, 128),
};

pub(super) static DARK_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn is_dark() -> bool {
    DARK_ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn colors() -> &'static ThemePalette {
    if is_dark() { &DARK } else { &LIGHT }
}

#[cfg(windows)]
pub use super::theme_windows::{
    background_brush, control_background, handle_message, initialize_window, set_mode,
    surface_brush,
};

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
