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
