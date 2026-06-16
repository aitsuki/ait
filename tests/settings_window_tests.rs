use ait::config::AppSettings;
use ait::ui::settings_window::{settings_window_center_position, SettingsViewModel};

#[test]
fn settings_view_model_hides_api_key_value() {
    let mut settings = AppSettings::default();
    settings.openai.encrypted_api_key = Some("encrypted-secret".to_string());

    let vm = SettingsViewModel::from(&settings);

    assert!(vm.has_openai_key);
    assert!(!format!("{vm:?}").contains("encrypted-secret"));
}

#[test]
fn settings_window_center_position_uses_work_area_center() {
    assert_eq!(
        settings_window_center_position((100, 50, 2020, 1130), (520, 360)),
        (800, 410)
    );
}

#[cfg(windows)]
#[test]
fn settings_window_allows_existing_window_class() {
    use ait::ui::settings_window::can_continue_after_register_class;
    use windows::Win32::Foundation::ERROR_CLASS_ALREADY_EXISTS;

    assert!(can_continue_after_register_class(
        0,
        ERROR_CLASS_ALREADY_EXISTS
    ));
}
