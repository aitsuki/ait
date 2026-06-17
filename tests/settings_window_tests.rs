use ait::config::{AppSettings, TranslatorProvider};
use ait::ui::settings_window::{
    SettingsEditAction, SettingsProfileDetailUpdate, SettingsViewModel, apply_settings_detail_update,
    apply_settings_edit_action,
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

    assert!(vm.profiles.iter().any(|item| item.label == "Google（默认）"));
    assert!(vm.profiles.iter().any(|item| item.label == "DeepSeek"));
    assert_eq!(vm.selected_profile.id, "deepseek");
    assert_eq!(vm.selected_profile.provider, TranslatorProvider::DeepSeek);
    assert!(vm.selected_profile.network_fields_enabled);
}

#[test]
fn settings_view_model_marks_default_and_builtin_profiles() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from(&settings);

    let google = vm.profiles.iter().find(|item| item.id == "google").unwrap();
    assert_eq!(google.label, "Google（默认）");
    let openai = vm.profiles.iter().find(|item| item.id == "openai").unwrap();
    assert_eq!(openai.label, "OpenAI");
}

#[test]
fn settings_view_model_does_not_show_builtin_label() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from(&settings);

    let google = vm.profiles.iter().find(|item| item.id == "google").unwrap();
    assert_eq!(google.label, "Google（默认）");
    let openai = vm.profiles.iter().find(|item| item.id == "openai").unwrap();
    assert_eq!(openai.label, "OpenAI");
    assert!(!vm.profiles.iter().any(|item| item.label.contains("内置")));
}

#[test]
fn google_profile_detail_is_readonly_and_hides_network_fields() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from_settings_with_selected(&settings, "google");

    assert_eq!(vm.selected_profile.id, "google");
    assert!(!vm.selected_profile.can_delete);
    assert!(!vm.selected_profile.name_editable);
    assert!(!vm.selected_profile.network_fields_visible);
    assert!(vm.selected_profile.google_notice_visible);
}

#[test]
fn non_google_profile_detail_is_editable_and_shows_network_fields() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from_settings_with_selected(&settings, "openai");

    assert_eq!(vm.selected_profile.id, "openai");
    assert!(!vm.selected_profile.can_delete);
    assert!(vm.selected_profile.name_editable);
    assert!(vm.selected_profile.network_fields_visible);
    assert!(!vm.selected_profile.google_notice_visible);
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
fn settings_detail_update_saves_selected_profile_fields() {
    let mut settings = AppSettings::default();
    let id = settings.add_custom_profile().id;

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: id.clone(),
            name: "Work GPT".to_string(),
            provider: TranslatorProvider::OpenAi,
            base_url: "https://example.test/v1".to_string(),
            model: "gpt-test".to_string(),
            api_key: Some("secret".to_string()),
            timeout_secs: 45,
            hotkey: "Ctrl+Alt+T".to_string(),
            copy_wait_ms: 250,
        },
    )
    .unwrap();

    let profile = settings.profile_by_id(&id).unwrap();
    assert_eq!(profile.name, "Work GPT");
    assert_eq!(profile.provider, TranslatorProvider::OpenAi);
    assert_eq!(profile.base_url, "https://example.test/v1");
    assert_eq!(profile.model, "gpt-test");
    assert_eq!(profile.encrypted_api_key.as_deref(), Some("secret"));
    assert_eq!(profile.timeout_secs, 45);
    assert_eq!(settings.hotkey, "Ctrl+Alt+T");
    assert_eq!(settings.clipboard_capture.copy_wait_ms, 250);
}

#[test]
fn settings_detail_update_clears_network_fields_for_google() {
    let mut settings = AppSettings::default();
    let google = settings.profile_by_id_mut("google").unwrap();
    google.base_url = "https://example.test".to_string();
    google.model = "model".to_string();
    google.encrypted_api_key = Some("secret".to_string());
    google.timeout_secs = 99;

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "google".to_string(),
            name: "Google".to_string(),
            provider: TranslatorProvider::Google,
            base_url: "https://ignored.test".to_string(),
            model: "ignored".to_string(),
            api_key: Some("ignored".to_string()),
            timeout_secs: 30,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
        },
    )
    .unwrap();

    let google = settings.profile_by_id("google").unwrap();
    assert_eq!(google.base_url, "");
    assert_eq!(google.model, "");
    assert_eq!(google.encrypted_api_key, None);
    assert_eq!(google.timeout_secs, 0);
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
