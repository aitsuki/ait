use ait::translator::{ProviderKind, TranslationErrorKind, TranslationRequest};

#[test]
fn translation_request_reports_text_length_without_text() {
    let request = TranslationRequest {
        text: "secret source text".to_string(),
        source_lang: "auto".to_string(),
        target_lang: "zh-CN".to_string(),
    };

    assert_eq!(request.text_len(), 18);
    assert!(!format!("{request:?}").contains("secret source text"));
}

#[test]
fn provider_kind_names_are_stable_for_logs() {
    assert_eq!(ProviderKind::GoogleFree.as_log_name(), "google_free");
    assert_eq!(
        ProviderKind::OpenAiCompatible.as_log_name(),
        "openai_compatible"
    );
}

#[test]
fn error_kind_user_messages_are_actionable() {
    assert!(TranslationErrorKind::RateLimited
        .user_message()
        .contains("稍后重试"));
    assert!(TranslationErrorKind::ProviderChanged
        .user_message()
        .contains("切换"));
}
