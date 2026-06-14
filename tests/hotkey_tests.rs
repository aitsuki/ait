use ait::hotkey::{Hotkey, KeyCode, Modifiers};

#[test]
fn parses_default_hotkey() {
    let hotkey = "Ctrl+Alt+E".parse::<Hotkey>().unwrap();

    assert_eq!(
        hotkey.modifiers,
        Modifiers {
            ctrl: true,
            alt: true,
            shift: false,
            win: false
        }
    );
    assert_eq!(hotkey.key, KeyCode::Char('E'));
}

#[test]
fn rejects_shortcut_without_non_modifier_key() {
    let err = "Ctrl+Alt".parse::<Hotkey>().unwrap_err().to_string();
    assert!(err.contains("必须包含一个普通按键"));
}

#[test]
fn normalizes_display_text() {
    let hotkey = " shift + ctrl + k ".parse::<Hotkey>().unwrap();

    assert_eq!(hotkey.to_string(), "Ctrl+Shift+K");
}
