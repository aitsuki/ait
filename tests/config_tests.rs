use ait::config::{AppSettings, SettingsStore, TranslatorProvider};

#[test]
fn defaults_include_builtin_translator_profiles() {
    let settings = AppSettings::default();

    assert_eq!(settings.default_profile_id, "google");
    let names: Vec<_> = settings
        .translator_profiles
        .iter()
        .map(|profile| profile.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["Google", "OpenAI", "Claude", "Gemini", "DeepSeek"]
    );
    assert_eq!(
        settings
            .translator_profiles
            .iter()
            .map(|profile| profile.provider)
            .collect::<Vec<_>>(),
        vec![
            TranslatorProvider::Google,
            TranslatorProvider::OpenAi,
            TranslatorProvider::Claude,
            TranslatorProvider::Gemini,
            TranslatorProvider::DeepSeek,
        ]
    );
    assert!(settings.profile_by_id("google").unwrap().built_in);
    assert_eq!(settings.hotkey, "Ctrl+Alt+E");
    assert!(settings.clipboard_capture.enabled);
    assert!(settings.clipboard_capture.open_manual_input_on_failure);
    assert!(!settings.markdown.render_enabled);
}

#[test]
fn google_profile_does_not_require_network_fields() {
    let google = AppSettings::default()
        .profile_by_id("google")
        .unwrap()
        .clone();

    assert_eq!(google.provider, TranslatorProvider::Google);
    assert_eq!(google.base_url, "");
    assert_eq!(google.model, "");
    assert_eq!(google.encrypted_api_key, None);
    assert_eq!(google.timeout_secs, 0);
}

#[test]
fn save_and_load_round_trips_settings() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().to_path_buf());
    let mut settings = AppSettings::default();
    let openai = settings.profile_by_id_mut("openai").unwrap();
    openai.base_url = "https://example.test/v1".to_string();
    openai.model = "test-model".to_string();

    store.save(&settings).unwrap();
    let loaded = store.load().unwrap();
    let loaded_openai = loaded.profile_by_id("openai").unwrap();

    assert_eq!(loaded_openai.base_url, "https://example.test/v1");
    assert_eq!(loaded_openai.model, "test-model");
}

#[test]
fn corrupted_config_is_backed_up_and_defaults_are_returned() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().to_path_buf());
    std::fs::create_dir_all(dir.path()).unwrap();
    std::fs::write(dir.path().join("settings.json"), "{ bad json").unwrap();

    let loaded = store.load().unwrap();

    assert_eq!(loaded, AppSettings::default());
    let backups: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("settings.json.bak.")
        })
        .collect();
    assert_eq!(backups.len(), 1);
}

#[test]
fn old_google_default_settings_migrate_to_google_profile() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().to_path_buf());
    std::fs::create_dir_all(dir.path()).unwrap();
    std::fs::write(
        dir.path().join("settings.json"),
        r#"{
          "default_provider": "google_free",
          "hotkey": "Ctrl+Alt+X",
          "target_language": "ja",
          "openai": {
            "name": "Existing OpenAI",
            "base_url": "https://example.test/v1",
            "encrypted_api_key": "encrypted-old-key",
            "model": "old-model",
            "timeout_secs": 45
          },
          "clipboard_capture": {
            "enabled": true,
            "open_manual_input_on_failure": false,
            "copy_wait_ms": 500
          },
          "window": { "width": 700, "height": 500 },
          "markdown": { "render_enabled": true }
        }"#,
    )
    .unwrap();

    let loaded = store.load().unwrap();

    assert_eq!(loaded.default_profile_id, "google");
    let openai = loaded.profile_by_id("openai").unwrap();
    assert_eq!(openai.name, "Existing OpenAI");
    assert_eq!(openai.base_url, "https://example.test/v1");
    assert_eq!(openai.encrypted_api_key.as_deref(), Some("encrypted-old-key"));
    assert_eq!(openai.model, "old-model");
    assert_eq!(openai.timeout_secs, 45);
    assert_eq!(loaded.hotkey, "Ctrl+Alt+X");
    assert_eq!(loaded.target_language, "ja");
    assert!(loaded.markdown.render_enabled);
}

#[test]
fn old_openai_default_with_key_migrates_default_to_openai_profile() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().to_path_buf());
    std::fs::create_dir_all(dir.path()).unwrap();
    std::fs::write(
        dir.path().join("settings.json"),
        r#"{
          "default_provider": "open_ai_compatible",
          "hotkey": "Ctrl+Alt+E",
          "target_language": "zh-CN",
          "openai": {
            "name": "Work Key",
            "base_url": "https://api.openai.com/v1",
            "encrypted_api_key": "encrypted-work-key",
            "model": "gpt-4o-mini",
            "timeout_secs": 30
          },
          "clipboard_capture": {
            "enabled": true,
            "open_manual_input_on_failure": true,
            "copy_wait_ms": 300
          },
          "window": { "width": 620, "height": 420 },
          "markdown": { "render_enabled": false }
        }"#,
    )
    .unwrap();

    let loaded = store.load().unwrap();

    assert_eq!(loaded.default_profile_id, "openai");
    assert_eq!(
        loaded
            .profile_by_id("openai")
            .unwrap()
            .encrypted_api_key
            .as_deref(),
        Some("encrypted-work-key")
    );
}

#[test]
fn google_profile_cannot_be_deleted() {
    let mut settings = AppSettings::default();

    let err = settings.delete_profile("google").unwrap_err();

    assert!(err.to_string().contains("内置翻译配置不能删除"));
    assert!(settings.profile_by_id("google").is_some());
}

#[test]
fn deleting_default_profile_selects_first_remaining_profile() {
    let mut settings = AppSettings::default();
    settings
        .translator_profiles
        .push(ait::config::TranslatorProfile {
            id: "custom-work".to_string(),
            name: "Work".to_string(),
            provider: TranslatorProvider::Custom,
            built_in: false,
            base_url: "https://example.test/v1".to_string(),
            model: "work-model".to_string(),
            encrypted_api_key: Some("encrypted".to_string()),
            timeout_secs: 20,
        });
    settings.default_profile_id = "custom-work".to_string();

    settings.delete_profile("custom-work").unwrap();

    assert_eq!(settings.default_profile_id, "google");
    assert!(settings.profile_by_id("custom-work").is_none());
}

#[test]
fn new_custom_profile_uses_unique_id_and_defaults() {
    let mut settings = AppSettings::default();

    let created = settings.add_custom_profile();

    assert_eq!(created.provider, TranslatorProvider::Custom);
    assert_eq!(created.name, "自定义配置");
    assert_eq!(created.base_url, "https://api.openai.com/v1");
    assert_eq!(created.model, "gpt-4o-mini");
    assert_eq!(created.timeout_secs, 30);
    assert!(settings.profile_by_id(&created.id).is_some());
}
