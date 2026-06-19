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

#[test]
fn parses_numeric_hotkey() {
    let hotkey = "Ctrl+Shift+1".parse::<Hotkey>().unwrap();

    assert_eq!(
        hotkey.modifiers,
        Modifiers {
            ctrl: true,
            alt: false,
            shift: true,
            win: false
        }
    );
    assert_eq!(hotkey.key, KeyCode::Char('1'));
    assert_eq!(hotkey.to_string(), "Ctrl+Shift+1");
}

#[test]
fn rejects_shortcut_without_modifier() {
    let err = "E".parse::<Hotkey>().unwrap_err().to_string();

    assert!(err.contains("至少包含一个修饰键"));
}
