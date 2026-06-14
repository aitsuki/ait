use ait::secret::SecretStore;

#[test]
#[cfg(windows)]
fn dpapi_protect_unprotect_round_trips() {
    let store = SecretStore::new("ait-test");
    let encrypted = store.protect("sk-test-secret").unwrap();

    assert_ne!(encrypted, "sk-test-secret");
    assert_eq!(store.unprotect(&encrypted).unwrap(), "sk-test-secret");
}

#[test]
#[cfg(not(windows))]
fn secret_store_is_windows_only() {
    let store = SecretStore::new("ait-test");

    assert!(store.protect("secret").is_err());
}
