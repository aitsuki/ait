use ait::config::AppSettings;
use ait::ui::settings_window::SettingsViewModel;

#[test]
fn settings_view_model_hides_api_key_value() {
    let mut settings = AppSettings::default();
    settings.openai.encrypted_api_key = Some("encrypted-secret".to_string());

    let vm = SettingsViewModel::from(&settings);

    assert!(vm.has_openai_key);
    assert!(!format!("{vm:?}").contains("encrypted-secret"));
}
