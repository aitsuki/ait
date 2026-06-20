use ait::config::{AppSettings, TranslatorProvider};
use ait::ui::settings_window::{
    SettingsApiKeyUpdate, SettingsEditAction, SettingsProfileDetailControl,
    SettingsProfileDetailUpdate, SettingsSaveOutcome, SettingsViewModel, api_key_placeholder_text,
    app_version_text, apply_settings_detail_update, apply_settings_edit_action,
    hotkey_capture_text, settings_api_key_input_text, settings_api_key_update_from_input,
    settings_edit_child_rect, settings_profile_detail_control_rect,
    settings_profile_detail_control_states, settings_profile_detail_hidden_rect,
    settings_profile_google_notice_text, settings_save_outcome_after_success,
    settings_static_controls_have_border, settings_static_text_uses_window_background,
    settings_window_center_position, settings_window_layout, settings_window_uses_background_brush,
};
use ait::update::latest_release_url;

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
fn saved_api_key_uses_fixed_placeholder_text() {
    assert_eq!(api_key_placeholder_text(), "********");
    assert_eq!(settings_api_key_input_text(true), "********");
    assert_eq!(settings_api_key_input_text(false), "");
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
fn settings_view_model_includes_auto_start_state() {
    let settings = AppSettings::default();

    let disabled =
        SettingsViewModel::from_settings_with_selected_and_auto_start(&settings, "google", false);
    let enabled =
        SettingsViewModel::from_settings_with_selected_and_auto_start(&settings, "google", true);

    assert!(!disabled.auto_start_enabled);
    assert!(enabled.auto_start_enabled);
}

#[test]
fn settings_view_model_includes_version_text() {
    let settings = AppSettings::default();

    let vm = SettingsViewModel::from(&settings);

    assert_eq!(vm.version_text, app_version_text());
    assert!(vm.version_text.starts_with("ait v"));
}

#[test]
fn settings_view_model_includes_update_state_defaults() {
    let settings = AppSettings::default();
    let vm = SettingsViewModel::from_settings_with_update_state(
        &settings,
        "google",
        true,
        false,
        latest_release_url().to_string(),
    );

    assert!(!vm.update_check_available);
    assert_eq!(vm.latest_release_url, latest_release_url());
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
            height: 60,
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
            api_key: SettingsApiKeyUpdate::Replace("secret".to_string()),
            timeout_secs: 45,
            hotkey: "Ctrl+Alt+T".to_string(),
            copy_wait_ms: 250,
            auto_start_enabled: false,
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
fn settings_detail_update_preserves_api_key_for_placeholder_input() {
    let mut settings = AppSettings::default();
    let openai = settings.profile_by_id_mut("openai").unwrap();
    openai.encrypted_api_key = Some("encrypted-old".to_string());

    let update = settings_api_key_update_from_input(
        openai.encrypted_api_key.clone(),
        api_key_placeholder_text(),
    );

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            provider: TranslatorProvider::OpenAi,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: update,
            timeout_secs: 30,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: false,
        },
    )
    .unwrap();

    assert_eq!(
        settings
            .profile_by_id("openai")
            .unwrap()
            .encrypted_api_key
            .as_deref(),
        Some("encrypted-old")
    );
}

#[test]
fn settings_detail_update_clears_api_key_for_empty_input() {
    let mut settings = AppSettings::default();
    let openai = settings.profile_by_id_mut("openai").unwrap();
    openai.encrypted_api_key = Some("encrypted-old".to_string());
    let existing_api_key = openai.encrypted_api_key.clone();

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            provider: TranslatorProvider::OpenAi,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: settings_api_key_update_from_input(existing_api_key, ""),
            timeout_secs: 30,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: false,
        },
    )
    .unwrap();

    assert_eq!(
        settings.profile_by_id("openai").unwrap().encrypted_api_key,
        None
    );
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
            api_key: SettingsApiKeyUpdate::Preserve,
            timeout_secs: 30,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: false,
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
            api_key: SettingsApiKeyUpdate::Replace("ignored".to_string()),
            timeout_secs: 30,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: false,
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
fn settings_detail_update_normalizes_hotkey_before_saving() {
    let mut settings = AppSettings::default();

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "google".to_string(),
            name: "Google".to_string(),
            provider: TranslatorProvider::Google,
            base_url: String::new(),
            model: String::new(),
            api_key: SettingsApiKeyUpdate::Preserve,
            timeout_secs: 0,
            hotkey: " shift + ctrl + 1 ".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: false,
        },
    )
    .unwrap();

    assert_eq!(settings.hotkey, "Ctrl+Shift+1");
}

#[test]
fn settings_detail_update_rejects_invalid_hotkey() {
    let mut settings = AppSettings::default();

    let err = apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "google".to_string(),
            name: "Google".to_string(),
            provider: TranslatorProvider::Google,
            base_url: String::new(),
            model: String::new(),
            api_key: SettingsApiKeyUpdate::Preserve,
            timeout_secs: 0,
            hotkey: "not-a-hotkey".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: false,
        },
    )
    .unwrap_err();

    assert!(err.to_string().contains("快捷键"));
    assert_eq!(settings.hotkey, "Ctrl+Alt+E");
}

#[test]
fn settings_detail_update_carries_auto_start_state_without_storing_in_app_settings() {
    let mut settings = AppSettings::default();

    apply_settings_detail_update(
        &mut settings,
        SettingsProfileDetailUpdate {
            id: "google".to_string(),
            name: "Google".to_string(),
            provider: TranslatorProvider::Google,
            base_url: String::new(),
            model: String::new(),
            api_key: SettingsApiKeyUpdate::Preserve,
            timeout_secs: 0,
            hotkey: "Ctrl+Alt+E".to_string(),
            copy_wait_ms: 300,
            auto_start_enabled: true,
        },
    )
    .unwrap();

    assert_eq!(settings.hotkey, "Ctrl+Alt+E");
}

#[test]
fn settings_detail_update_can_carry_enabled_auto_start_choice() {
    let update = SettingsProfileDetailUpdate {
        id: "google".to_string(),
        name: "Google".to_string(),
        provider: TranslatorProvider::Google,
        base_url: String::new(),
        model: String::new(),
        api_key: SettingsApiKeyUpdate::Preserve,
        timeout_secs: 0,
        hotkey: "Ctrl+Alt+E".to_string(),
        copy_wait_ms: 300,
        auto_start_enabled: true,
    };

    assert!(update.auto_start_enabled);
}

#[test]
fn hotkey_capture_text_formats_supported_combinations() {
    let ctrl_alt = ait::hotkey::Modifiers {
        ctrl: true,
        alt: true,
        shift: false,
        win: false,
    };
    let ctrl_shift = ait::hotkey::Modifiers {
        ctrl: true,
        alt: false,
        shift: true,
        win: false,
    };

    assert_eq!(
        hotkey_capture_text(0x54, ctrl_alt).as_deref(),
        Some("Ctrl+Alt+T")
    );
    assert_eq!(
        hotkey_capture_text(0x31, ctrl_shift).as_deref(),
        Some("Ctrl+Shift+1")
    );
    assert_eq!(
        hotkey_capture_text(0x70, ctrl_alt).as_deref(),
        Some("Ctrl+Alt+F1")
    );
    assert_eq!(
        hotkey_capture_text(0x87, ctrl_alt).as_deref(),
        Some("Ctrl+Alt+F24")
    );
}

#[test]
fn hotkey_capture_text_ignores_incomplete_or_unsupported_keys() {
    let none = ait::hotkey::Modifiers {
        ctrl: false,
        alt: false,
        shift: false,
        win: false,
    };
    let ctrl = ait::hotkey::Modifiers {
        ctrl: true,
        alt: false,
        shift: false,
        win: false,
    };

    assert_eq!(hotkey_capture_text(0x54, none), None);
    assert_eq!(hotkey_capture_text(0x11, ctrl), None);
    assert_eq!(hotkey_capture_text(0x1B, ctrl), None);
    assert_eq!(hotkey_capture_text(0xBA, ctrl), None);
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
fn settings_window_layout_places_auto_start_with_global_settings() {
    let layout = settings_window_layout();

    assert!(layout.auto_start.y > layout.hotkey.y);
    assert!(layout.auto_start.y < layout.separator.y);
    assert!(layout.version.y > layout.profile_list.y);
    assert!(layout.update_action.y >= layout.version.y);
}

#[test]
fn app_version_text_uses_v0_1_6() {
    assert_eq!(app_version_text(), "ait v0.1.6");
}

#[test]
fn settings_window_layout_keeps_version_label_in_visible_client_area() {
    let layout = settings_window_layout();

    assert!(layout.version.y + layout.version.height <= 410);
}

#[test]
fn settings_window_erases_hidden_control_pixels() {
    assert!(settings_window_uses_background_brush());
}

#[test]
fn settings_static_controls_are_not_framed() {
    assert!(!settings_static_controls_have_border());
}

#[test]
fn settings_static_text_controls_use_window_background() {
    assert!(settings_static_text_uses_window_background(0));
    assert!(settings_static_text_uses_window_background(3110));
    assert!(settings_static_text_uses_window_background(3111));
    assert!(settings_static_text_uses_window_background(3118));
    assert!(!settings_static_text_uses_window_background(3102));
}

#[test]
fn settings_edit_controls_use_modern_border() {
    for id in [3102, 3104, 3105, 3106, 3107, 3108] {
        assert!(!ait::ui::edit::edit_uses_native_border(id));
    }
}

#[test]
fn settings_auto_start_checkbox_uses_modern_drawing() {
    assert!(ait::ui::checkbox::is_modern_checkbox(3117));
    assert!(!ait::ui::checkbox::checkbox_uses_native_border(3117));
}

#[test]
fn settings_modern_edit_controls_leave_room_for_parent_drawn_border() {
    let layout = settings_window_layout();
    let rect = settings_edit_child_rect(
        3108,
        layout.hotkey.x,
        layout.hotkey.y,
        layout.hotkey.width,
        layout.hotkey.height,
    );

    assert_eq!(layout.hotkey.height, 32);
    assert_eq!(rect.x, 122);
    assert_eq!(rect.y, 22);
    assert_eq!(rect.width, 172);
    assert_eq!(rect.height, 24);
}

#[test]
fn settings_profile_detail_layout_gives_single_line_edits_full_content_height() {
    let name = settings_profile_detail_control_rect(SettingsProfileDetailControl::NameInput);
    let base_url = settings_profile_detail_control_rect(SettingsProfileDetailControl::BaseUrlInput);
    let model = settings_profile_detail_control_rect(SettingsProfileDetailControl::ModelInput);
    let api_key = settings_profile_detail_control_rect(SettingsProfileDetailControl::ApiKeyInput);
    let timeout = settings_profile_detail_control_rect(SettingsProfileDetailControl::TimeoutInput);

    for rect in [name, base_url, model, api_key, timeout] {
        assert_eq!(rect.height, 32);
        assert_eq!(
            settings_edit_child_rect(3102, rect.x, rect.y, rect.width, rect.height).height,
            24
        );
    }
    assert!(base_url.y - name.y >= 44);
    assert!(model.y - base_url.y >= 44);
    assert!(api_key.y - model.y >= 44);
    assert!(timeout.y - api_key.y >= 44);
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
