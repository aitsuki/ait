use ait::translator::openai_compatible::{OpenAiCompatibleConfig, OpenAiCompatibleTranslator};
use ait::translator::{ProviderKind, TranslationRequest, Translator};
use httpmock::Method::POST;
use httpmock::MockServer;
use serde_json::json;

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
        provider: ProviderKind::OpenAi,
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
    assert_eq!(response.provider, ProviderKind::OpenAi);
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
        provider: ProviderKind::OpenAi,
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

#[tokio::test]
async fn deepseek_requests_disable_thinking() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/chat/completions")
            .json_body_includes(r#"{"thinking":{"type":"disabled"}}"#);
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"choices":[{"message":{"content":"你好"}}]}"#);
    });
    let translator = OpenAiCompatibleTranslator::new(OpenAiCompatibleConfig {
        provider: ProviderKind::DeepSeek,
        base_url: server.url("/v1"),
        api_key: "sk-test".to_string(),
        model: "deepseek-v4-flash".to_string(),
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
    assert_eq!(response.provider, ProviderKind::DeepSeek);
    assert_eq!(response.translated_text, "你好");
}

#[tokio::test]
async fn openai_compatible_requests_omit_deepseek_thinking_field() {
    let server = MockServer::start();
    let expected_body = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "system",
                "content": "Translate the user's text into zh-CN. Return only the translation."
            },
            {
                "role": "user",
                "content": "hello"
            }
        ],
        "temperature": 0.2
    });
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/chat/completions")
            .json_body(expected_body);
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"choices":[{"message":{"content":"你好"}}]}"#);
    });
    let translator = OpenAiCompatibleTranslator::new(OpenAiCompatibleConfig {
        provider: ProviderKind::OpenAi,
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
    assert_eq!(response.provider, ProviderKind::OpenAi);
    assert_eq!(response.translated_text, "你好");
}
