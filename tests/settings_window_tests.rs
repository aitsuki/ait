use ait::config::{AppSettings, TranslatorProvider};
use ait::ui::settings_window::{
    SettingsEditAction, SettingsProfileDetailControl, SettingsProfileDetailUpdate,
    SettingsSaveOutcome, SettingsViewModel, apply_settings_detail_update,
    apply_settings_edit_action, settings_profile_detail_control_rect,
    settings_profile_detail_control_states, settings_profile_detail_hidden_rect,
    settings_profile_google_notice_text, settings_save_outcome_after_success,
    settings_static_controls_have_border, settings_window_center_position, settings_window_layout,
    settings_window_uses_background_brush,
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

    assert!(
        vm.profiles
            .iter()
            .any(|item| item.label == "Google（默认）")
    );
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
    let mut settings = AppSettings::default();
    let google = settings.profile_by_id_mut("google").unwrap();
    google.base_url = "https://api.deepseek.com/v1".to_string();
    google.model = "deepseek-chat".to_string();
    google.encrypted_api_key = Some("stored-key".to_string());
    google.timeout_secs = 30;

    let vm = SettingsViewModel::from_settings_with_selected(&settings, "google");

    assert_eq!(vm.selected_profile.id, "google");
    assert_eq!(vm.selected_profile.base_url, "");
    assert_eq!(vm.selected_profile.model, "");
    assert!(!vm.selected_profile.has_api_key);
    assert_eq!(vm.selected_profile.timeout_secs, 0);
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
fn profile_detail_control_states_hide_google_network_labels_and_inputs() {
    let settings = AppSettings::default();
    let google = SettingsViewModel::from_settings_with_selected(&settings, "google");
    let states = settings_profile_detail_control_states(&google.selected_profile);

    for control in [
        SettingsProfileDetailControl::NameLabel,
        SettingsProfileDetailControl::NameInput,
        SettingsProfileDetailControl::BaseUrlLabel,
        SettingsProfileDetailControl::BaseUrlInput,
        SettingsProfileDetailControl::ModelLabel,
        SettingsProfileDetailControl::ModelInput,
        SettingsProfileDetailControl::ApiKeyLabel,
        SettingsProfileDetailControl::ApiKeyInput,
        SettingsProfileDetailControl::TimeoutLabel,
        SettingsProfileDetailControl::TimeoutInput,
    ] {
        let state = states
            .iter()
            .find(|state| state.control == control)
            .unwrap();
        assert!(!state.visible, "{control:?} should be hidden for Google");
    }

    let notice = states
        .iter()
        .find(|state| state.control == SettingsProfileDetailControl::GoogleNotice)
        .unwrap();
    assert!(notice.visible);
}

#[test]
fn profile_detail_control_states_show_non_google_network_labels_and_inputs() {
    let settings = AppSettings::default();
    let deepseek = SettingsViewModel::from_settings_with_selected(&settings, "deepseek");
    let states = settings_profile_detail_control_states(&deepseek.selected_profile);

    for control in [
        SettingsProfileDetailControl::NameLabel,
        SettingsProfileDetailControl::NameInput,
        SettingsProfileDetailControl::BaseUrlLabel,
        SettingsProfileDetailControl::BaseUrlInput,
        SettingsProfileDetailControl::ModelLabel,
        SettingsProfileDetailControl::ModelInput,
        SettingsProfileDetailControl::ApiKeyLabel,
        SettingsProfileDetailControl::ApiKeyInput,
        SettingsProfileDetailControl::TimeoutLabel,
        SettingsProfileDetailControl::TimeoutInput,
    ] {
        let state = states
            .iter()
            .find(|state| state.control == control)
            .unwrap();
        assert!(state.visible, "{control:?} should be visible for DeepSeek");
    }

    let notice = states
        .iter()
        .find(|state| state.control == SettingsProfileDetailControl::GoogleNotice)
        .unwrap();
    assert!(!notice.visible);
}

#[test]
fn hidden_profile_detail_controls_have_no_visible_layout_area() {
    let hidden = settings_profile_detail_hidden_rect();

    assert_eq!(hidden.width, 0);
    assert_eq!(hidden.height, 0);
    assert!(hidden.x < 0);
    assert!(hidden.y < 0);
    assert_ne!(
        settings_profile_detail_control_rect(SettingsProfileDetailControl::BaseUrlInput),
        hidden
    );
    assert_ne!(
        settings_profile_detail_control_rect(SettingsProfileDetailControl::GoogleNotice),
        hidden
    );
}

#[test]
fn google_notice_uses_top_detail_position() {
    assert_eq!(
        settings_profile_detail_control_rect(SettingsProfileDetailControl::GoogleNotice),
        ait::ui::settings_window::SettingsControlRect {
            x: 266,
            y: 100,
            width: 420,
            height: 44,
        }
    );
}

#[test]
fn google_notice_explains_no_network_fields_are_needed() {
    assert_eq!(
        settings_profile_google_notice_text(),
        "Google 使用内置免 Key 翻译，无需填写 Base URL、模型或 API Key。"
    );
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
    settings.clipboard_capture.copy_wait_ms = 425;
    let id = settings.add_custom_profile().id;
    settings.profile_by_id_mut(&id).unwrap().provider = TranslatorProvider::OpenAi;

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
    assert_eq!(settings.clipboard_capture.copy_wait_ms, 425);
}

#[test]
fn settings_detail_update_preserves_existing_provider() {
    let mut settings = AppSettings::default();
    let id = settings.add_custom_profile().id;
    settings.profile_by_id_mut(&id).unwrap().provider = TranslatorProvider::DeepSeek;

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: id.clone(),
            name: "DeepSeek Work".to_string(),
            provider: TranslatorProvider::Google,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: None,
            timeout_secs: 30,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
        },
    )
    .unwrap();

    assert_eq!(
        settings.profile_by_id(&id).unwrap().provider,
        TranslatorProvider::DeepSeek
    );
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

#[test]
fn successful_settings_save_keeps_window_open() {
    assert_eq!(
        settings_save_outcome_after_success(),
        SettingsSaveOutcome::KeepOpen
    );
}

#[test]
fn settings_window_layout_places_global_settings_above_profiles() {
    let layout = settings_window_layout();

    assert!(layout.hotkey.y < layout.separator.y);
    assert!(layout.profile_list.y > layout.separator.y);
    assert!(layout.name.y > layout.separator.y);
}

#[test]
fn settings_window_erases_hidden_control_pixels() {
    assert!(settings_window_uses_background_brush());
}

#[test]
fn settings_static_controls_are_not_framed() {
    assert!(!settings_static_controls_have_border());
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
