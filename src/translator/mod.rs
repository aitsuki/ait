pub mod google_free;
pub mod openai_compatible;

use crate::error::Result;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    GoogleFree,
    OpenAiCompatible,
}

impl ProviderKind {
    pub fn as_log_name(self) -> &'static str {
        match self {
            Self::GoogleFree => "google_free",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }
}

#[derive(Clone)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

impl TranslationRequest {
    pub fn text_len(&self) -> usize {
        self.text.chars().count()
    }
}

impl fmt::Debug for TranslationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationRequest")
            .field("text_len", &self.text_len())
            .field("source_lang", &self.source_lang)
            .field("target_lang", &self.target_lang)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationResponse {
    pub translated_text: String,
    pub provider: ProviderKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationErrorKind {
    Unauthorized,
    RateLimited,
    Timeout,
    Network,
    ProviderChanged,
    InvalidResponse,
}

impl TranslationErrorKind {
    pub fn user_message(self) -> &'static str {
        match self {
            Self::Unauthorized => "接口认证失败，请检查 API Key。",
            Self::RateLimited => "翻译服务暂时限流，请稍后重试，或切换到其他翻译提供方。",
            Self::Timeout => "翻译请求超时，请重试。",
            Self::Network => "网络连接失败，请检查网络或代理设置。",
            Self::ProviderChanged => "内置翻译接口可能已变化，请重试或切换到 OpenAI 兼容接口。",
            Self::InvalidResponse => "翻译服务返回了无法识别的数据。",
        }
    }
}

pub trait Translator: Send + Sync {
    fn translate<'a>(
        &'a self,
        request: TranslationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<TranslationResponse>> + Send + 'a>>;
}
