use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    GoogleFree,
    OpenAiCompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiSettings {
    pub name: String,
    pub base_url: String,
    pub encrypted_api_key: Option<String>,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardCaptureSettings {
    pub enabled: bool,
    pub open_manual_input_on_failure: bool,
    pub copy_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowSettings {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownSettings {
    pub render_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_provider: ProviderKind,
    pub hotkey: String,
    pub target_language: String,
    pub openai: OpenAiSettings,
    pub clipboard_capture: ClipboardCaptureSettings,
    pub window: WindowSettings,
    pub markdown: MarkdownSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_provider: ProviderKind::GoogleFree,
            hotkey: "Ctrl+Alt+E".to_string(),
            target_language: "zh-CN".to_string(),
            openai: OpenAiSettings {
                name: "OpenAI Compatible".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                encrypted_api_key: None,
                model: "gpt-4o-mini".to_string(),
                timeout_secs: 30,
            },
            clipboard_capture: ClipboardCaptureSettings {
                enabled: true,
                open_manual_input_on_failure: true,
                copy_wait_ms: 300,
            },
            window: WindowSettings {
                width: 620,
                height: 420,
            },
            markdown: MarkdownSettings {
                render_enabled: false,
            },
        }
    }
}

pub struct SettingsStore {
    dir: PathBuf,
}

impl SettingsStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_dir() -> Result<PathBuf> {
        let project_dirs = directories::ProjectDirs::from("dev", "aitsu", "ait")
            .ok_or_else(|| AppError::Config("无法定位配置目录".to_string()))?;
        Ok(project_dirs.config_dir().to_path_buf())
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    pub fn load(&self) -> Result<AppSettings> {
        let path = self.path();
        if !path.exists() {
            return Ok(AppSettings::default());
        }

        let raw = fs::read_to_string(&path)?;
        match serde_json::from_str::<AppSettings>(&raw) {
            Ok(settings) => Ok(settings),
            Err(_) => {
                self.backup_corrupt_file(&path)?;
                Ok(AppSettings::default())
            }
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        let raw = serde_json::to_string_pretty(settings)?;
        fs::write(self.path(), raw)?;
        Ok(())
    }

    fn backup_corrupt_file(&self, path: &Path) -> Result<()> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| AppError::Config(err.to_string()))?
            .as_secs();
        let backup = self.dir.join(format!("settings.json.bak.{ts}"));
        fs::rename(path, backup)?;
        Ok(())
    }
}
