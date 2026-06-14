use ait::translator::openai_compatible::{OpenAiCompatibleConfig, OpenAiCompatibleTranslator};
use ait::translator::{ProviderKind, TranslationRequest, Translator};
use httpmock::Method::POST;
use httpmock::MockServer;

#[tokio::test]
async fn sends_chat_completions_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/chat/completions")
            .header("authorization", "Bearer sk-test");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"choices":[{"message":{"content":"你好"}}]}"#);
    });
    let translator = OpenAiCompatibleTranslator::new(OpenAiCompatibleConfig {
        base_url: server.url("/v1"),
        api_key: "sk-test".to_string(),
        model: "test-model".to_string(),
        timeout_secs: 10,
    })
    .unwrap();

    let response = translator
        .translate(TranslationRequest {
            text: "hello".to_string(),
            source_lang: "auto".to_string(),
            target_lang: "zh-CN".to_string(),
        })
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.provider, ProviderKind::OpenAiCompatible);
    assert_eq!(response.translated_text, "你好");
}

#[tokio::test]
async fn maps_unauthorized_response() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/chat/completions");
        then.status(401).body("unauthorized");
    });
    let translator = OpenAiCompatibleTranslator::new(OpenAiCompatibleConfig {
        base_url: server.url("/v1"),
        api_key: "bad-key".to_string(),
        model: "test-model".to_string(),
        timeout_secs: 10,
    })
    .unwrap();

    let err = translator
        .translate(TranslationRequest {
            text: "hello".to_string(),
            source_lang: "auto".to_string(),
            target_lang: "zh-CN".to_string(),
        })
        .await
        .unwrap_err()
        .to_string();

    assert!(err.contains("认证失败"));
}
