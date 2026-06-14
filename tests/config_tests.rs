use ait::config::{AppSettings, ProviderKind, SettingsStore};

#[test]
fn defaults_use_google_free_and_ctrl_alt_e() {
    let settings = AppSettings::default();

    assert_eq!(settings.default_provider, ProviderKind::GoogleFree);
    assert_eq!(settings.hotkey, "Ctrl+Alt+E");
    assert!(settings.clipboard_capture.enabled);
    assert!(settings.clipboard_capture.open_manual_input_on_failure);
    assert!(!settings.markdown.render_enabled);
}

#[test]
fn save_and_load_round_trips_settings() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().to_path_buf());
    let mut settings = AppSettings::default();
    settings.openai.base_url = "https://example.test/v1".to_string();
    settings.openai.model = "test-model".to_string();

    store.save(&settings).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded.openai.base_url, "https://example.test/v1");
    assert_eq!(loaded.openai.model, "test-model");
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
