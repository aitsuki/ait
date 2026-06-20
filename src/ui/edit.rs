#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditVisualState {
    pub focused: bool,
    pub readonly: bool,
    pub disabled: bool,
}

impl EditVisualState {
    pub fn normal() -> Self {
        Self {
            focused: false,
            readonly: false,
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
pub struct EditPalette {
    pub background: RgbColor,
    pub border: RgbColor,
    pub text: RgbColor,
}

pub fn edit_palette(state: EditVisualState) -> EditPalette {
    if state.disabled {
        return EditPalette {
            background: RgbColor::new(243, 244, 246),
            border: RgbColor::new(209, 213, 219),
            text: RgbColor::new(156, 163, 175),
        };
    }

    if state.readonly {
        return EditPalette {
            background: RgbColor::new(248, 250, 252),
            border: RgbColor::new(203, 213, 225),
            text: RgbColor::new(31, 41, 55),
        };
    }

    EditPalette {
        background: RgbColor::new(255, 255, 255),
        border: if state.focused {
            RgbColor::new(37, 99, 235)
        } else {
            RgbColor::new(203, 213, 225)
        },
        text: RgbColor::new(31, 41, 55),
    }
}

pub fn edit_kind_for_control(id: usize) -> Option<EditKind> {
    match id {
        2101 | 2102 => Some(EditKind::MultiLine),
        3102 | 3104 | 3105 | 3106 | 3107 | 3108 => Some(EditKind::SingleLine),
        _ => None,
    }
}

pub fn is_modern_edit(id: usize) -> bool {
    edit_kind_for_control(id).is_some()
}

pub fn edit_uses_native_border(id: usize) -> bool {
    !is_modern_edit(id)
}

#[cfg(test)]
mod tests {
    use super::{
        EditKind, EditVisualState, RgbColor, edit_kind_for_control, edit_palette,
        edit_uses_native_border, is_modern_edit,
    };

    #[test]
    fn maps_translation_multiline_edits() {
        assert_eq!(edit_kind_for_control(2101), Some(EditKind::MultiLine));
        assert_eq!(edit_kind_for_control(2102), Some(EditKind::MultiLine));
    }

    #[test]
    fn maps_settings_single_line_edits() {
        for id in [3102, 3104, 3105, 3106, 3107, 3108] {
            assert_eq!(edit_kind_for_control(id), Some(EditKind::SingleLine));
            assert!(is_modern_edit(id));
        }
    }

    #[test]
    fn ignores_unknown_controls() {
        assert_eq!(edit_kind_for_control(9999), None);
        assert!(!is_modern_edit(9999));
        assert!(edit_uses_native_border(9999));
    }

    #[test]
    fn normal_edit_uses_white_surface() {
        let palette = edit_palette(EditVisualState::normal());
        assert_eq!(palette.background, RgbColor::new(255, 255, 255));
        assert_eq!(palette.border, RgbColor::new(203, 213, 225));
        assert_eq!(palette.text, RgbColor::new(31, 41, 55));
    }

    #[test]
    fn focused_edit_uses_blue_border() {
        let palette = edit_palette(EditVisualState {
            focused: true,
            ..EditVisualState::normal()
        });
        assert_eq!(palette.border, RgbColor::new(37, 99, 235));
    }

    #[test]
    fn readonly_edit_is_distinct_from_disabled() {
        let readonly = edit_palette(EditVisualState {
            readonly: true,
            ..EditVisualState::normal()
        });
        let disabled = edit_palette(EditVisualState {
            disabled: true,
            ..EditVisualState::normal()
        });

        assert_eq!(readonly.background, RgbColor::new(248, 250, 252));
        assert_eq!(readonly.text, RgbColor::new(31, 41, 55));
        assert_eq!(disabled.background, RgbColor::new(243, 244, 246));
        assert_eq!(disabled.text, RgbColor::new(156, 163, 175));
        assert_ne!(readonly, disabled);
    }
}
