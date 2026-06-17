use ait::config::{AppSettings, TranslatorProvider};
use ait::ui::settings_window::{
    SettingsEditAction, SettingsViewModel, apply_settings_edit_action,
    settings_window_center_position,
};

#[test]
fn settings_view_model_hides_api_key_value() {
    let mut settings = AppSettings::default();
    settings
        .profile_by_id_mut("openai")
        .unwrap()
        .encrypted_api_key = Some("encrypted-secret".to_string());
    settings.default_profile_id = "openai".to_string();

    let vm = SettingsViewModel::from(&settings);

    assert_eq!(vm.selected_profile.id, "openai");
    assert!(vm.selected_profile.has_api_key);
    assert!(!format!("{vm:?}").contains("encrypted-secret"));
}

#[test]
fn settings_view_model_lists_profiles_and_selected_detail() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from_settings_with_selected(&settings, "deepseek");

    assert!(vm.profiles.iter().any(|item| item.label == "Google"));
    assert!(vm.profiles.iter().any(|item| item.label == "DeepSeek"));
    assert_eq!(vm.selected_profile.id, "deepseek");
    assert_eq!(vm.selected_profile.provider, TranslatorProvider::DeepSeek);
    assert!(vm.selected_profile.network_fields_enabled);
}

#[test]
fn settings_edit_action_creates_and_selects_custom_profile() {
    let mut settings = AppSettings::default();

    let id = apply_settings_edit_action(&mut settings, SettingsEditAction::NewProfile).unwrap();

    let created = settings.profile_by_id(&id).unwrap();
    assert_eq!(created.provider, TranslatorProvider::Custom);
    assert!(!created.built_in);
}

#[test]
fn settings_edit_action_rejects_missing_profile_selection() {
    let mut settings = AppSettings::default();

    let err = apply_settings_edit_action(
        &mut settings,
        SettingsEditAction::SelectProfile("missing".to_string()),
    )
    .unwrap_err();

    assert!(err.to_string().contains("翻译配置不存在"));
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
