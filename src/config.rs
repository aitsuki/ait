use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslatorProvider {
    Google,
    OpenAi,
    Claude,
    Gemini,
    DeepSeek,
    Custom,
}

impl TranslatorProvider {
    #[allow(non_upper_case_globals)]
    pub const GoogleFree: Self = Self::Google;
    #[allow(non_upper_case_globals)]
    pub const OpenAiCompatible: Self = Self::OpenAi;

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Google => "Google",
            Self::OpenAi => "OpenAI",
            Self::Claude => "Claude",
            Self::Gemini => "Gemini",
            Self::DeepSeek => "DeepSeek",
            Self::Custom => "自定义",
        }
    }

    pub fn as_log_name(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::OpenAi => "openai",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
            Self::DeepSeek => "deepseek",
            Self::Custom => "custom",
        }
    }
}

pub type ProviderKind = TranslatorProvider;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslatorProfile {
    pub id: String,
    pub name: String,
    pub provider: TranslatorProvider,
    pub built_in: bool,
    pub base_url: String,
    pub model: String,
    pub encrypted_api_key: Option<String>,
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
    pub default_profile_id: String,
    pub translator_profiles: Vec<TranslatorProfile>,
    pub hotkey: String,
    pub target_language: String,
    pub clipboard_capture: ClipboardCaptureSettings,
    pub window: WindowSettings,
    pub markdown: MarkdownSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_profile_id: "google".to_string(),
            translator_profiles: builtin_translator_profiles(),
            hotkey: "Ctrl+Alt+E".to_string(),
            target_language: "zh-CN".to_string(),
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

pub fn builtin_translator_profiles() -> Vec<TranslatorProfile> {
    vec![
        TranslatorProfile {
            id: "google".to_string(),
            name: "Google".to_string(),
            provider: TranslatorProvider::Google,
            built_in: true,
            base_url: String::new(),
            model: String::new(),
            encrypted_api_key: None,
            timeout_secs: 0,
        },
        TranslatorProfile {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            provider: TranslatorProvider::OpenAi,
            built_in: true,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            encrypted_api_key: None,
            timeout_secs: 30,
        },
        TranslatorProfile {
            id: "claude".to_string(),
            name: "Claude".to_string(),
            provider: TranslatorProvider::Claude,
            built_in: true,
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-5-haiku-latest".to_string(),
            encrypted_api_key: None,
            timeout_secs: 30,
        },
        TranslatorProfile {
            id: "gemini".to_string(),
            name: "Gemini".to_string(),
            provider: TranslatorProvider::Gemini,
            built_in: true,
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            model: "gemini-1.5-flash".to_string(),
            encrypted_api_key: None,
            timeout_secs: 30,
        },
        TranslatorProfile {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            provider: TranslatorProvider::DeepSeek,
            built_in: true,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            encrypted_api_key: None,
            timeout_secs: 30,
        },
    ]
}

impl AppSettings {
    pub fn profile_by_id(&self, id: &str) -> Option<&TranslatorProfile> {
        self.translator_profiles
            .iter()
            .find(|profile| profile.id == id)
    }

    pub fn profile_by_id_mut(&mut self, id: &str) -> Option<&mut TranslatorProfile> {
        self.translator_profiles
            .iter_mut()
            .find(|profile| profile.id == id)
    }

    pub fn default_profile(&self) -> Result<&TranslatorProfile> {
        self.profile_by_id(&self.default_profile_id)
            .or_else(|| self.translator_profiles.first())
            .ok_or_else(|| AppError::Config("没有可用的翻译配置".to_string()))
    }

    pub fn normalized(mut self) -> Self {
        for builtin in builtin_translator_profiles() {
            if self.profile_by_id(&builtin.id).is_none() {
                self.translator_profiles.push(builtin);
            }
        }
        if self.profile_by_id(&self.default_profile_id).is_none() {
            self.default_profile_id = self
                .translator_profiles
                .first()
                .map(|profile| profile.id.clone())
                .unwrap_or_else(|| "google".to_string());
        }
        self
    }

    pub fn add_custom_profile(&mut self) -> TranslatorProfile {
        let id = self.next_custom_profile_id();
        let profile = TranslatorProfile {
            id,
            name: "自定义配置".to_string(),
            provider: TranslatorProvider::Custom,
            built_in: false,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            encrypted_api_key: None,
            timeout_secs: 30,
        };
        self.translator_profiles.push(profile.clone());
        profile
    }

    pub fn delete_profile(&mut self, id: &str) -> Result<()> {
        let profile = self
            .profile_by_id(id)
            .ok_or_else(|| AppError::Config("翻译配置不存在".to_string()))?;
        if profile.built_in {
            return Err(AppError::Config("内置翻译配置不能删除".to_string()));
        }
        self.translator_profiles.retain(|profile| profile.id != id);
        if self.default_profile_id == id {
            self.default_profile_id = self
                .translator_profiles
                .first()
                .map(|profile| profile.id.clone())
                .unwrap_or_else(|| "google".to_string());
        }
        Ok(())
    }

    pub fn set_default_profile(&mut self, id: &str) -> Result<()> {
        if self.profile_by_id(id).is_none() {
            return Err(AppError::Config("翻译配置不存在".to_string()));
        }
        self.default_profile_id = id.to_string();
        Ok(())
    }

    fn next_custom_profile_id(&self) -> String {
        let mut index = 1;
        loop {
            let id = if index == 1 {
                "custom".to_string()
            } else {
                format!("custom-{index}")
            };
            if self.profile_by_id(&id).is_none() {
                return id;
            }
            index += 1;
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
