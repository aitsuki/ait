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

impl AppError {
    pub fn user_summary(&self) -> String {
        match self {
            AppError::Hotkey(_) => "快捷键注册失败，请在设置中更换快捷键。".to_string(),
            AppError::Capture(_) => "没有取到选中文本，可以手动粘贴文本后重试。".to_string(),
            AppError::Translate(msg) if msg.contains("API Key 缺失") => {
                "翻译失败：API Key 缺失，请在设置中填写 API Key。".to_string()
            }
            AppError::Translate(msg) if msg.starts_with("翻译服务返回了无法识别的数据。") => {
                "翻译服务返回了无法识别的数据。".to_string()
            }
            AppError::Translate(msg) => format!("翻译失败：{msg}"),
            AppError::Network(_) => "网络连接失败，请检查网络或代理设置后重试。".to_string(),
            AppError::Secret(_) => "API Key 读取失败，请重新保存接口配置。".to_string(),
            AppError::Config(_) => "配置读取失败，已尝试恢复默认配置。".to_string(),
            AppError::Windows(_) | AppError::Io(_) | AppError::Json(_) => self.to_string(),
        }
    }
}
