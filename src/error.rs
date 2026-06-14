use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(String),
    #[error("密钥存储错误: {0}")]
    Secret(String),
    #[error("快捷键错误: {0}")]
    Hotkey(String),
    #[error("取词错误: {0}")]
    Capture(String),
    #[error("翻译错误: {0}")]
    Translate(String),
    #[error("Windows API 错误: {0}")]
    Windows(String),
    #[error("网络错误: {0}")]
    Network(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
}
