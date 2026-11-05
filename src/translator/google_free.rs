use crate::error::{AppError, Result};
use crate::translator::{
    ProviderKind, TranslationErrorKind, TranslationRequest, TranslationResponse, Translator,
    invalid_response_error, request_error, response_snippet,
};
use reqwest::StatusCode;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub struct GoogleFreeTranslator {
    client: reqwest::Client,
}

impl Default for GoogleFreeTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleFreeTranslator {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .user_agent("ait/0.1")
                .build()
                .expect("reqwest client"),
        }
    }

    async fn fetch(&self, url: &str) -> reqwest::Result<(StatusCode, String, String)> {
        let mut retries = 0;
        loop {
            debug_assert!(retries <= 2);
            let result: reqwest::Result<_> = async {
                let response = self.client.get(url).send().await?.error_for_status()?;
                let content_type = response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();
                Ok((response.status(), content_type, response.text().await?))
            }
            .await;
            match result {
                Err(ref err) if retries < 2 && is_retryable(err) => {
                    tokio::time::sleep(Duration::from_millis(500 << retries)).await;
                    retries += 1;
                }
                _ => return result,
            }
        }
    }

    async fn translate_inner(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let url = format!(
            "https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&tl={}&dt=t&q={}",
            urlencoding::encode(&request.source_lang),
            urlencoding::encode(&request.target_lang),
            urlencoding::encode(&request.text),
        );
        let (status, content_type, body) =
            self.fetch(&url).await.map_err(|err| match err.status() {
                Some(StatusCode::TOO_MANY_REQUESTS) => AppError::Translate(
                    TranslationErrorKind::RateLimited.user_message().to_string(),
                ),
                Some(StatusCode::FORBIDDEN) => AppError::Translate(
                    TranslationErrorKind::ProviderChanged
                        .user_message()
                        .to_string(),
                ),
                Some(status) => {
                    AppError::Translate(format!("内置 Google 翻译失败，状态码: {status}"))
                }
                None => request_error(err),
            })?;
        if !status.is_success() {
            return Err(AppError::Translate(format!(
                "内置 Google 翻译失败，状态码: {status}"
            )));
        }
        let json: Value = serde_json::from_str(&body).map_err(|err| {
            invalid_response_error(format!(
                "响应不是 JSON；content-type: {content_type}；片段: {}；解析错误: {err}",
                response_snippet(&body)
            ))
        })?;
        let translated = parse_google_response(&json)?;

        Ok(TranslationResponse {
            translated_text: translated,
            provider: ProviderKind::Google,
        })
    }
}

impl Translator for GoogleFreeTranslator {
    fn translate<'a>(
        &'a self,
        request: TranslationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<TranslationResponse>> + Send + 'a>> {
        Box::pin(self.translate_inner(request))
    }
}

fn is_retryable(err: &reqwest::Error) -> bool {
    err.is_connect()
        || err.is_timeout()
        || err.is_body()
        || err.status().is_some_and(|status| {
            status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
        })
}

fn parse_google_response(json: &Value) -> Result<String> {
    let segments = json
        .get(0)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response_error("JSON 顶层缺少第 0 项数组"))?;

    let mut out = String::new();
    for segment in segments {
        let text = segment
            .get(0)
            .and_then(Value::as_str)
            .ok_or_else(|| invalid_response_error("翻译段缺少第 0 项字符串"))?;
        out.push_str(text);
    }

    if out.trim().is_empty() {
        return Err(invalid_response_error("解析出的译文为空"));
    }
    Ok(out)
}
