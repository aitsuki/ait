#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboVisualState {
    pub focused: bool,
    pub dropped: bool,
    pub disabled: bool,
}

impl ComboVisualState {
    pub fn normal() -> Self {
        Self {
            focused: false,
            dropped: false,
            disabled: false,
        }
    }
}

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboTextRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub fn combo_palette(state: ComboVisualState) -> ComboPalette {
    if state.disabled {
        return ComboPalette {
            background: RgbColor::new(243, 244, 246),
            border: RgbColor::new(209, 213, 219),
            text: RgbColor::new(156, 163, 175),
        };
    }

    ComboPalette {
        background: RgbColor::new(255, 255, 255),
        border: if state.focused || state.dropped {
            RgbColor::new(37, 99, 235)
        } else {
            RgbColor::new(203, 213, 225)
        },
        text: RgbColor::new(31, 41, 55),
    }
}

pub fn is_modern_combo(id: usize) -> bool {
    id == 2106
}

pub fn combo_uses_native_border(id: usize) -> bool {
    !is_modern_combo(id)
}

pub fn modern_combo_frame_rect(left: i32, top: i32, right: i32, bottom: i32) -> ComboTextRect {
    ComboTextRect {
        left,
        top,
        right,
        bottom,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComboVisualState, RgbColor, combo_palette, combo_uses_native_border, is_modern_combo,
        modern_combo_frame_rect,
    };

    #[test]
    fn maps_translation_profile_combo() {
        assert!(is_modern_combo(2106));
    }

    #[test]
    fn ignores_unknown_controls() {
        assert!(!is_modern_combo(9999));
        assert!(combo_uses_native_border(9999));
    }

    #[test]
    fn modern_combo_does_not_use_native_border() {
        assert!(!combo_uses_native_border(2106));
    }

    #[test]
    fn normal_combo_uses_white_surface() {
        let palette = combo_palette(ComboVisualState::normal());
        assert_eq!(palette.background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.border, RgbColor::new(203, 213, 225));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn focused_combo_uses_blue_border() {
        let palette = combo_palette(ComboVisualState {
            focused: true,
            ..ComboVisualState::normal()
        });
        assert_eq!(palette.border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn dropped_combo_uses_active_border() {
        let palette = combo_palette(ComboVisualState {
            dropped: true,
            ..ComboVisualState::normal()
        });
        assert_eq!(palette.border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn disabled_combo_uses_muted_colors() {
        let palette = combo_palette(ComboVisualState {
            disabled: true,
            ..ComboVisualState::normal()
        });
        assert_eq!(palette.background, RgbColor::new(243, 244, 246));
        assert_eq!(palette.border, RgbColor::new(209, 213, 219));
        assert_eq!(palette.text, RgbColor::new(156, 163, 175));
    }

    #[test]
    fn frame_rect_matches_control_bounds() {
        let rect = modern_combo_frame_rect(408, 12, 588, 38);
        assert_eq!(rect.left, 408);
        assert_eq!(rect.top, 12);
        assert_eq!(rect.right, 588);
        assert_eq!(rect.bottom, 38);
    }
}
