use ait::translator::google_free::GoogleFreeTranslator;
use ait::translator::{
    ProviderKind, TranslationErrorKind, TranslationRequest, TranslationResponse, Translator,
};
use httpmock::Method::GET;
use httpmock::MockServer;

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
    assert_eq!(ProviderKind::Google.as_log_name(), "google");
    assert_eq!(ProviderKind::OpenAi.as_log_name(), "openai");
    assert_eq!(ProviderKind::Claude.as_log_name(), "claude");
    assert_eq!(ProviderKind::Gemini.as_log_name(), "gemini");
    assert_eq!(ProviderKind::DeepSeek.as_log_name(), "deepseek");
    assert_eq!(ProviderKind::Custom.as_log_name(), "custom");
}

#[test]
fn error_kind_user_messages_are_actionable() {
    assert!(
        TranslationErrorKind::RateLimited
            .user_message()
            .contains("稍后重试")
    );
    assert!(
        TranslationErrorKind::ProviderChanged
            .user_message()
            .contains("切换")
    );
}

#[tokio::test]
async fn google_free_translates_array_response() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/translate_a/single")
            .query_param("client", "gtx")
            .query_param("sl", "auto")
            .query_param("tl", "zh-CN")
            .query_param("dt", "t")
            .query_param("q", "hello");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"[[["你好","hello",null,null,1]],null,"en"]"#);
    });
    let translator = GoogleFreeTranslator::with_base_url(server.url(""));

    let response = translator
        .translate(ait::translator::TranslationRequest {
            text: "hello".to_string(),
            source_lang: "auto".to_string(),
            target_lang: "zh-CN".to_string(),
        })
        .await
        .unwrap();

    mock.assert();
    assert_eq!(
        response,
        TranslationResponse {
            translated_text: "你好".to_string(),
            provider: ait::translator::ProviderKind::GoogleFree,
        }
    );
}

#[tokio::test]
async fn google_free_maps_rate_limit() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/translate_a/single");
        then.status(429).body("too many requests");
    });
    let translator = GoogleFreeTranslator::with_base_url(server.url(""));

    let err = translator
        .translate(ait::translator::TranslationRequest {
            text: "hello".to_string(),
            source_lang: "auto".to_string(),
            target_lang: "zh-CN".to_string(),
        })
        .await
        .unwrap_err()
        .to_string();

    assert!(err.contains("限流"));
}
