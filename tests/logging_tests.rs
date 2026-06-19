#[test]
fn safe_text_len_counts_chars_without_exposing_text() {
    assert_eq!(ait::logging::safe_text_len("hello世界"), 7);
}

#[test]
fn log_dir_uses_logs_subdirectory() {
    let dir = ait::logging::log_dir().unwrap();

    assert_eq!(dir.file_name().and_then(|name| name.to_str()), Some("logs"));
}
