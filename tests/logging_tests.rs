#[test]
fn safe_text_len_counts_chars_without_exposing_text() {
    assert_eq!(ait::logging::safe_text_len("hello世界"), 7);
}
