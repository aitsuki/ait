# Selection Translation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not use `superpowers:subagent-driven-development`; this repository's `AGENTS.md` forbids it.

**Goal:** Build the first Windows-only MVP of the lightweight tray-based selection translator described in `docs/superpowers/specs/2026-06-15-selection-translation-design.md`.

**Architecture:** Create a Rust + Win32 application with small modules for app lifecycle, hotkeys, clipboard capture, translation providers, config, DPAPI secrets, logging, and UI. The first working slice should register `Ctrl+Alt+E`, capture selected text via clipboard copy, translate with the built-in unofficial Google provider by default, and show the result in a lightweight native window.

**Tech Stack:** Rust stable, `windows-rs`, Win32 APIs, `serde`, `reqwest`, `tokio`, DPAPI, Windows clipboard APIs, `tracing`, unit tests plus focused manual Windows verification.

---

## Ground Rules

- Follow `AGENTS.md`: docs produced by `superpowers` specs are Chinese; this plan may be English, but implementation-facing comments and user-facing text should be Chinese unless the code convention later says otherwise.
- Do not implement UI Automation, OCR, history, streaming output, WebView, Electron, Qt, Flutter, Anthropic, Gemini, official Google Cloud Translation, or arbitrary HTTP templates.
- Default provider is built-in `GoogleFreeTranslator`, an unofficial no-key Google Translate endpoint. Treat it as unstable and show actionable fallback errors.
- OpenAI-compatible provider is optional and user-configured.
- Clipboard capture only promises to restore text clipboard content, not files, images, or rich text.
- Do not log API keys or full source text. Log provider, status code, text length, and error kind.
- Prefer TDD for pure modules. For Win32 message loop/UI code, add compile checks and manual verification steps.
- Commit after each task that leaves the app in a coherent state.

## File Structure

Create this structure:

```text
Cargo.toml
src/
  main.rs
  app.rs
  command.rs
  error.rs
  logging.rs
  config.rs
  secret.rs
  hotkey.rs
  capture.rs
  translator/
    mod.rs
    google_free.rs
    openai_compatible.rs
  ui/
    mod.rs
    tray.rs
    translate_window.rs
    settings_window.rs
tests/
  config_tests.rs
  hotkey_tests.rs
  translator_google_tests.rs
  translator_openai_tests.rs
  capture_tests.rs
docs/
  manual-test-checklists/
    windows-mvp.md
```

Responsibilities:

- `main.rs`: program entry, calls `app::run()`.
- `app.rs`: single-instance check, config load, logging init, tray/window/hotkey wiring, Win32 message loop.
- `command.rs`: typed app commands such as `TranslateSelection`, `OpenSettings`, `Exit`.
- `error.rs`: shared error enums and user-facing error summaries.
- `logging.rs`: local log initialization and privacy-safe helpers.
- `config.rs`: `settings.json` model, defaults, load/save, corruption backup.
- `secret.rs`: DPAPI protect/unprotect API-key wrapper.
- `hotkey.rs`: fixed hotkey parsing and Win32 `RegisterHotKey` wrapper.
- `capture.rs`: clipboard text snapshot, copy trigger, wait/read/restore flow.
- `translator/mod.rs`: provider trait, shared request/response types, provider selection.
- `translator/google_free.rs`: unofficial Google Translate no-key adapter.
- `translator/openai_compatible.rs`: OpenAI-compatible chat completions adapter.
- `ui/*`: native tray, translation window, and minimal settings window.
- `tests/*`: pure and integration-like tests using mocks or dependency seams.

## Task 1: Initialize Rust Project And Dependencies

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

- [x] **Step 1: Initialize the Rust package**

Run:

```powershell
cargo init --bin --name ait .
```

Expected: `Cargo.toml` and `src/main.rs` are created.

- [x] **Step 2: Add dependencies**

Run:

```powershell
cargo add serde --features derive
cargo add serde_json thiserror tracing tracing-subscriber tracing-appender directories urlencoding
cargo add tokio --features rt-multi-thread,macros,time
cargo add reqwest --features json,rustls-tls
cargo add windows --features Win32_Foundation,Win32_System_Com,Win32_System_DataExchange,Win32_System_Memory,Win32_System_Threading,Win32_Security_Cryptography,Win32_UI_Input_KeyboardAndMouse,Win32_UI_Shell,Win32_UI_WindowsAndMessaging,Win32_Graphics_Gdi
cargo add --dev tempfile httpmock
```

Expected: Cargo resolves dependencies and updates `Cargo.toml`.

- [x] **Step 3: Create library module skeleton**

Modify `src/lib.rs`:

```rust
pub mod app;
pub mod capture;
pub mod command;
pub mod config;
pub mod error;
pub mod hotkey;
pub mod logging;
pub mod secret;
pub mod translator;
pub mod ui;
```

Modify `src/main.rs`:

```rust
fn main() -> ait::error::Result<()> {
    ait::app::run()
}
```

- [x] **Step 4: Create empty module files**

Create:

```text
src/app.rs
src/capture.rs
src/command.rs
src/config.rs
src/error.rs
src/hotkey.rs
src/logging.rs
src/secret.rs
src/translator/mod.rs
src/translator/google_free.rs
src/translator/openai_compatible.rs
src/ui/mod.rs
src/ui/tray.rs
src/ui/translate_window.rs
src/ui/settings_window.rs
```

Put temporary compile stubs in `src/app.rs`:

```rust
use crate::error::Result;

pub fn run() -> Result<()> {
    Ok(())
}
```

Put module exports in `src/translator/mod.rs`:

```rust
pub mod google_free;
pub mod openai_compatible;
```

Put module exports in `src/ui/mod.rs`:

```rust
pub mod settings_window;
pub mod translate_window;
pub mod tray;
```

- [x] **Step 5: Add shared error type**

Modify `src/error.rs`:

```rust
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
```

- [x] **Step 6: Verify compile**

Run:

```powershell
cargo test
```

Expected: build succeeds and reports zero or more tests passing, with no compile errors.

- [x] **Step 7: Commit**

```powershell
git add Cargo.toml Cargo.lock src
git commit -m "chore: initialize rust app"
```

## Task 2: Config Model, Defaults, And Corruption Recovery

**Files:**
- Modify: `src/config.rs`
- Modify: `src/error.rs`
- Test: `tests/config_tests.rs`

- [x] **Step 1: Write config tests**

Create `tests/config_tests.rs`:

```rust
use ait::config::{AppSettings, ProviderKind, SettingsStore};

#[test]
fn defaults_use_google_free_and_ctrl_alt_e() {
    let settings = AppSettings::default();

    assert_eq!(settings.default_provider, ProviderKind::GoogleFree);
    assert_eq!(settings.hotkey, "Ctrl+Alt+E");
    assert!(settings.clipboard_capture.enabled);
    assert!(settings.clipboard_capture.open_manual_input_on_failure);
    assert!(!settings.markdown.render_enabled);
}

#[test]
fn save_and_load_round_trips_settings() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().to_path_buf());
    let mut settings = AppSettings::default();
    settings.openai.base_url = "https://example.test/v1".to_string();
    settings.openai.model = "test-model".to_string();

    store.save(&settings).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded.openai.base_url, "https://example.test/v1");
    assert_eq!(loaded.openai.model, "test-model");
}

#[test]
fn corrupted_config_is_backed_up_and_defaults_are_returned() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().to_path_buf());
    std::fs::create_dir_all(dir.path()).unwrap();
    std::fs::write(dir.path().join("settings.json"), "{ bad json").unwrap();

    let loaded = store.load().unwrap();

    assert_eq!(loaded, AppSettings::default());
    let backups: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("settings.json.bak."))
        .collect();
    assert_eq!(backups.len(), 1);
}
```

- [x] **Step 2: Run tests and confirm they fail**

Run:

```powershell
cargo test --test config_tests
```

Expected: compile fails because `ait::config` types are not defined.

- [x] **Step 3: Implement config types and store**

Modify `src/config.rs`:

```rust
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
            window: WindowSettings { width: 620, height: 420 },
            markdown: MarkdownSettings { render_enabled: false },
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
```

- [x] **Step 4: Run config tests**

Run:

```powershell
cargo test --test config_tests
```

Expected: all 3 tests pass.

- [x] **Step 5: Commit**

```powershell
git add src/config.rs tests/config_tests.rs
git commit -m "feat: add settings store"
```

## Task 3: Hotkey Parser And Fixed Shortcut Model

**Files:**
- Modify: `src/hotkey.rs`
- Test: `tests/hotkey_tests.rs`

- [x] **Step 1: Write parser tests**

Create `tests/hotkey_tests.rs`:

```rust
use ait::hotkey::{Hotkey, KeyCode, Modifiers};

#[test]
fn parses_default_hotkey() {
    let hotkey = "Ctrl+Alt+E".parse::<Hotkey>().unwrap();

    assert_eq!(hotkey.modifiers, Modifiers { ctrl: true, alt: true, shift: false, win: false });
    assert_eq!(hotkey.key, KeyCode::Char('E'));
}

#[test]
fn rejects_shortcut_without_non_modifier_key() {
    let err = "Ctrl+Alt".parse::<Hotkey>().unwrap_err().to_string();
    assert!(err.contains("必须包含一个普通按键"));
}

#[test]
fn normalizes_display_text() {
    let hotkey = " shift + ctrl + k ".parse::<Hotkey>().unwrap();

    assert_eq!(hotkey.to_string(), "Ctrl+Shift+K");
}
```

- [x] **Step 2: Run parser tests and confirm failure**

Run:

```powershell
cargo test --test hotkey_tests
```

Expected: compile fails because hotkey types are missing.

- [x] **Step 3: Implement hotkey parser**

Modify `src/hotkey.rs`:

```rust
use crate::error::{AppError, Result};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Function(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    pub modifiers: Modifiers,
    pub key: KeyCode,
}

impl FromStr for Hotkey {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self> {
        let mut modifiers = Modifiers { ctrl: false, alt: false, shift: false, win: false };
        let mut key = None;

        for raw in input.split('+') {
            let part = raw.trim().to_ascii_lowercase();
            match part.as_str() {
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                "win" | "windows" | "super" => modifiers.win = true,
                "" => {}
                value if value.len() == 1 => {
                    key = Some(KeyCode::Char(value.chars().next().unwrap().to_ascii_uppercase()));
                }
                value if value.starts_with('f') => {
                    let number = value[1..]
                        .parse::<u8>()
                        .map_err(|_| AppError::Hotkey(format!("不支持的按键: {value}")))?;
                    if !(1..=24).contains(&number) {
                        return Err(AppError::Hotkey(format!("不支持的功能键: F{number}")));
                    }
                    key = Some(KeyCode::Function(number));
                }
                value => return Err(AppError::Hotkey(format!("不支持的按键: {value}"))),
            }
        }

        let key = key.ok_or_else(|| AppError::Hotkey("快捷键必须包含一个普通按键".to_string()))?;
        Ok(Self { modifiers, key })
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.modifiers.ctrl { parts.push("Ctrl".to_string()); }
        if self.modifiers.alt { parts.push("Alt".to_string()); }
        if self.modifiers.shift { parts.push("Shift".to_string()); }
        if self.modifiers.win { parts.push("Win".to_string()); }
        parts.push(match self.key {
            KeyCode::Char(ch) => ch.to_string(),
            KeyCode::Function(n) => format!("F{n}"),
        });
        write!(f, "{}", parts.join("+"))
    }
}
```

- [x] **Step 4: Add Win32 conversion skeleton behind Windows cfg**

Append to `src/hotkey.rs`:

```rust
#[cfg(windows)]
impl Hotkey {
    pub fn win32_modifiers(self) -> windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
        };
        let mut bits = HOT_KEY_MODIFIERS(0);
        if self.modifiers.ctrl { bits |= MOD_CONTROL; }
        if self.modifiers.alt { bits |= MOD_ALT; }
        if self.modifiers.shift { bits |= MOD_SHIFT; }
        if self.modifiers.win { bits |= MOD_WIN; }
        bits
    }

    pub fn win32_vk(self) -> u32 {
        match self.key {
            KeyCode::Char(ch) => ch as u32,
            KeyCode::Function(n) => 0x70 + (n as u32) - 1,
        }
    }
}
```

- [x] **Step 5: Run tests**

Run:

```powershell
cargo test --test hotkey_tests
cargo test
```

Expected: tests pass.

- [x] **Step 6: Commit**

```powershell
git add src/hotkey.rs tests/hotkey_tests.rs
git commit -m "feat: add fixed hotkey parser"
```

## Task 4: DPAPI Secret Wrapper

**Files:**
- Modify: `src/secret.rs`
- Test: `tests/secret_tests.rs`

- [x] **Step 1: Write DPAPI round-trip test**

Create `tests/secret_tests.rs`:

```rust
use ait::secret::SecretStore;

#[test]
#[cfg(windows)]
fn dpapi_protect_unprotect_round_trips() {
    let store = SecretStore::new("ait-test");
    let encrypted = store.protect("sk-test-secret").unwrap();

    assert_ne!(encrypted, "sk-test-secret");
    assert_eq!(store.unprotect(&encrypted).unwrap(), "sk-test-secret");
}

#[test]
#[cfg(not(windows))]
fn secret_store_is_windows_only() {
    let store = SecretStore::new("ait-test");

    assert!(store.protect("secret").is_err());
}
```

- [x] **Step 2: Run test and confirm failure**

Run:

```powershell
cargo test --test secret_tests
```

Expected: compile fails because `SecretStore` is missing.

- [x] **Step 3: Implement non-Windows guard**

Modify `src/secret.rs`:

```rust
use crate::error::{AppError, Result};

pub struct SecretStore {
    purpose: String,
}

impl SecretStore {
    pub fn new(purpose: impl Into<String>) -> Self {
        Self { purpose: purpose.into() }
    }

    #[cfg(not(windows))]
    pub fn protect(&self, _plain: &str) -> Result<String> {
        Err(AppError::Secret("DPAPI 仅支持 Windows".to_string()))
    }

    #[cfg(not(windows))]
    pub fn unprotect(&self, _encrypted: &str) -> Result<String> {
        Err(AppError::Secret("DPAPI 仅支持 Windows".to_string()))
    }
}
```

- [x] **Step 4: Implement Windows DPAPI**

Append to `src/secret.rs`:

```rust
#[cfg(windows)]
impl SecretStore {
    pub fn protect(&self, plain: &str) -> Result<String> {
        use windows::core::PCWSTR;
        use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
        use windows::Win32::System::Memory::LocalFree;

        let mut input = plain.as_bytes().to_vec();
        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let description: Vec<u16> = self.purpose.encode_utf16().chain(Some(0)).collect();
        let mut out_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            CryptProtectData(
                &mut in_blob,
                PCWSTR(description.as_ptr()),
                None,
                None,
                None,
                0,
                &mut out_blob,
            )
            .map_err(|err| AppError::Secret(format!("DPAPI 加密失败: {err}")))?;

            let bytes = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec();
            let _ = LocalFree(Some(out_blob.pbData as isize));
            Ok(base64_encode(&bytes))
        }
    }

    pub fn unprotect(&self, encrypted: &str) -> Result<String> {
        use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
        use windows::Win32::System::Memory::LocalFree;

        let mut input = base64_decode(encrypted)?;
        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_mut_ptr(),
        };
        let mut out_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            CryptUnprotectData(&mut in_blob, None, None, None, None, 0, &mut out_blob)
                .map_err(|err| AppError::Secret(format!("DPAPI 解密失败: {err}")))?;

            let bytes = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec();
            let _ = LocalFree(Some(out_blob.pbData as isize));
            String::from_utf8(bytes).map_err(|err| AppError::Secret(format!("密钥不是 UTF-8: {err}")))
        }
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(b2 & 0b0011_1111) as usize] as char } else { '=' });
    }
    out
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let cleaned = input.trim().as_bytes();
    if cleaned.len() % 4 != 0 {
        return Err(AppError::Secret("无效的 base64 密文长度".to_string()));
    }
    for chunk in cleaned.chunks(4) {
        let vals: Vec<u8> = chunk.iter().map(|b| match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 64,
            _ => 255,
        }).collect();
        if vals.iter().any(|v| *v == 255) {
            return Err(AppError::Secret("无效的 base64 字符".to_string()));
        }
        bytes.push((vals[0] << 2) | (vals[1] >> 4));
        if vals[2] != 64 {
            bytes.push((vals[1] << 4) | (vals[2] >> 2));
        }
        if vals[3] != 64 {
            bytes.push((vals[2] << 6) | vals[3]);
        }
    }
    Ok(bytes)
}
```

- [x] **Step 5: Run secret tests**

Run:

```powershell
cargo test --test secret_tests
```

Expected on Windows: DPAPI round-trip test passes.

- [x] **Step 6: Commit**

```powershell
git add src/secret.rs tests/secret_tests.rs
git commit -m "feat: protect api keys with dpapi"
```

## Task 5: Translator Trait And Privacy-Safe Errors

**Files:**
- Modify: `src/translator/mod.rs`
- Modify: `src/error.rs`
- Test: add to `tests/translator_google_tests.rs`

- [x] **Step 1: Write trait-level test**

Create `tests/translator_google_tests.rs`:

```rust
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
    assert_eq!(ProviderKind::OpenAiCompatible.as_log_name(), "openai_compatible");
}

#[test]
fn error_kind_user_messages_are_actionable() {
    assert!(TranslationErrorKind::RateLimited.user_message().contains("稍后重试"));
    assert!(TranslationErrorKind::ProviderChanged.user_message().contains("切换"));
}
```

- [x] **Step 2: Run test and confirm failure**

Run:

```powershell
cargo test --test translator_google_tests
```

Expected: compile fails because translator types are missing.

- [x] **Step 3: Implement translator shared types**

Modify `src/translator/mod.rs`:

```rust
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
```

- [x] **Step 4: Run tests**

Run:

```powershell
cargo test --test translator_google_tests
```

Expected: all 3 tests pass.

- [x] **Step 5: Commit**

```powershell
git add src/translator/mod.rs tests/translator_google_tests.rs
git commit -m "feat: add translator abstractions"
```

## Task 6: Built-In Google Free Translator Adapter

**Files:**
- Modify: `src/translator/google_free.rs`
- Modify: `tests/translator_google_tests.rs`

- [x] **Step 1: Add HTTP mock tests for Google adapter**

Append to `tests/translator_google_tests.rs`:

```rust
use ait::translator::google_free::GoogleFreeTranslator;
use ait::translator::{TranslationResponse, Translator};
use httpmock::Method::GET;
use httpmock::MockServer;

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

    let response = translator.translate(ait::translator::TranslationRequest {
        text: "hello".to_string(),
        source_lang: "auto".to_string(),
        target_lang: "zh-CN".to_string(),
    }).await.unwrap();

    mock.assert();
    assert_eq!(response, TranslationResponse {
        translated_text: "你好".to_string(),
        provider: ait::translator::ProviderKind::GoogleFree,
    });
}

#[tokio::test]
async fn google_free_maps_rate_limit() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/translate_a/single");
        then.status(429).body("too many requests");
    });
    let translator = GoogleFreeTranslator::with_base_url(server.url(""));

    let err = translator.translate(ait::translator::TranslationRequest {
        text: "hello".to_string(),
        source_lang: "auto".to_string(),
        target_lang: "zh-CN".to_string(),
    }).await.unwrap_err().to_string();

    assert!(err.contains("限流"));
}
```

- [x] **Step 2: Run tests and confirm failure**

Run:

```powershell
cargo test --test translator_google_tests
```

Expected: compile fails because `GoogleFreeTranslator` is missing.

- [x] **Step 3: Implement Google free adapter**

Modify `src/translator/google_free.rs`:

```rust
use crate::error::{AppError, Result};
use crate::translator::{
    ProviderKind, TranslationErrorKind, TranslationRequest, TranslationResponse, Translator,
};
use reqwest::StatusCode;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub struct GoogleFreeTranslator {
    client: reqwest::Client,
    base_url: String,
}

impl Default for GoogleFreeTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleFreeTranslator {
    pub fn new() -> Self {
        Self::with_base_url("https://translate.googleapis.com".to_string())
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .user_agent("ait/0.1")
                .build()
                .expect("reqwest client"),
            base_url,
        }
    }

    async fn translate_inner(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let url = format!("{}/translate_a/single", self.base_url.trim_end_matches('/'));
        let response = self.client
            .get(url)
            .query(&[
                ("client", "gtx"),
                ("sl", request.source_lang.as_str()),
                ("tl", request.target_lang.as_str()),
                ("dt", "t"),
                ("q", request.text.as_str()),
            ])
            .send()
            .await
            .map_err(|err| AppError::Network(err.to_string()))?;

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AppError::Translate(TranslationErrorKind::RateLimited.user_message().to_string()));
        }
        if status == StatusCode::FORBIDDEN {
            return Err(AppError::Translate(TranslationErrorKind::ProviderChanged.user_message().to_string()));
        }
        if !status.is_success() {
            return Err(AppError::Translate(format!("内置 Google 翻译失败，状态码: {status}")));
        }

        let json: Value = response.json().await.map_err(|_| {
            AppError::Translate(TranslationErrorKind::InvalidResponse.user_message().to_string())
        })?;
        let translated = parse_google_response(&json)?;

        Ok(TranslationResponse {
            translated_text: translated,
            provider: ProviderKind::GoogleFree,
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

fn parse_google_response(json: &Value) -> Result<String> {
    let segments = json
        .get(0)
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Translate(TranslationErrorKind::InvalidResponse.user_message().to_string()))?;

    let mut out = String::new();
    for segment in segments {
        let text = segment
            .get(0)
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::Translate(TranslationErrorKind::InvalidResponse.user_message().to_string()))?;
        out.push_str(text);
    }

    if out.trim().is_empty() {
        return Err(AppError::Translate(TranslationErrorKind::InvalidResponse.user_message().to_string()));
    }
    Ok(out)
}
```

- [x] **Step 4: Run Google translator tests**

Run:

```powershell
cargo test --test translator_google_tests
```

Expected: all Google translator tests pass.

- [x] **Step 5: Commit**

```powershell
git add src/translator/google_free.rs tests/translator_google_tests.rs
git commit -m "feat: add built-in google translator"
```

## Task 7: OpenAI-Compatible Translator Adapter

**Files:**
- Modify: `src/translator/openai_compatible.rs`
- Test: `tests/translator_openai_tests.rs`

- [x] **Step 1: Write OpenAI adapter tests**

Create `tests/translator_openai_tests.rs`:

```rust
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
    }).unwrap();

    let response = translator.translate(TranslationRequest {
        text: "hello".to_string(),
        source_lang: "auto".to_string(),
        target_lang: "zh-CN".to_string(),
    }).await.unwrap();

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
    }).unwrap();

    let err = translator.translate(TranslationRequest {
        text: "hello".to_string(),
        source_lang: "auto".to_string(),
        target_lang: "zh-CN".to_string(),
    }).await.unwrap_err().to_string();

    assert!(err.contains("认证失败"));
}
```

- [x] **Step 2: Run test and confirm failure**

Run:

```powershell
cargo test --test translator_openai_tests
```

Expected: compile fails because OpenAI adapter types are missing.

- [x] **Step 3: Implement OpenAI adapter**

Modify `src/translator/openai_compatible.rs`:

```rust
use crate::error::{AppError, Result};
use crate::translator::{
    ProviderKind, TranslationErrorKind, TranslationRequest, TranslationResponse, Translator,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OpenAiCompatibleConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

pub struct OpenAiCompatibleTranslator {
    client: reqwest::Client,
    config: OpenAiCompatibleConfig,
}

impl OpenAiCompatibleTranslator {
    pub fn new(config: OpenAiCompatibleConfig) -> Result<Self> {
        if config.api_key.trim().is_empty() {
            return Err(AppError::Translate("API Key 缺失".to_string()));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|err| AppError::Network(err.to_string()))?;
        Ok(Self { client, config })
    }

    async fn translate_inner(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: format!("Translate the user's text into {}. Return only the translation.", request.target_lang),
                },
                ChatMessage { role: "user".to_string(), content: request.text },
            ],
            temperature: 0.2,
        };

        let response = self.client
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|err| AppError::Network(err.to_string()))?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(AppError::Translate(TranslationErrorKind::Unauthorized.user_message().to_string()));
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AppError::Translate(TranslationErrorKind::RateLimited.user_message().to_string()));
        }
        if !status.is_success() {
            return Err(AppError::Translate(format!("OpenAI 兼容接口失败，状态码: {status}")));
        }

        let body: ChatResponse = response.json().await.map_err(|_| {
            AppError::Translate(TranslationErrorKind::InvalidResponse.user_message().to_string())
        })?;
        let text = body.choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| AppError::Translate(TranslationErrorKind::InvalidResponse.user_message().to_string()))?;

        Ok(TranslationResponse {
            translated_text: text,
            provider: ProviderKind::OpenAiCompatible,
        })
    }
}

impl Translator for OpenAiCompatibleTranslator {
    fn translate<'a>(
        &'a self,
        request: TranslationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<TranslationResponse>> + Send + 'a>> {
        Box::pin(self.translate_inner(request))
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}
```

- [x] **Step 4: Run OpenAI tests**

Run:

```powershell
cargo test --test translator_openai_tests
```

Expected: all OpenAI adapter tests pass.

- [x] **Step 5: Commit**

```powershell
git add src/translator/openai_compatible.rs tests/translator_openai_tests.rs
git commit -m "feat: add openai compatible translator"
```

## Task 8: Clipboard Capture Core And Windows Implementation

**Files:**
- Modify: `src/capture.rs`
- Test: `tests/capture_tests.rs`

- [x] **Step 1: Write dependency-seam tests**

Create `tests/capture_tests.rs`:

```rust
use ait::capture::{CaptureErrorKind, CaptureService, ClipboardBackend};
use std::cell::RefCell;
use std::time::Duration;

#[derive(Default)]
struct FakeClipboard {
    current: RefCell<Option<String>>,
    copied: RefCell<Option<String>>,
}

impl ClipboardBackend for FakeClipboard {
    fn read_text(&self) -> ait::error::Result<Option<String>> {
        Ok(self.current.borrow().clone())
    }

    fn write_text(&self, text: &str) -> ait::error::Result<()> {
        *self.current.borrow_mut() = Some(text.to_string());
        Ok(())
    }

    fn send_copy(&self) -> ait::error::Result<()> {
        if let Some(text) = self.copied.borrow().clone() {
            *self.current.borrow_mut() = Some(text);
        }
        Ok(())
    }
}

#[test]
fn capture_restores_previous_text_clipboard() {
    let fake = FakeClipboard::default();
    *fake.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.copied.borrow_mut() = Some("selected text".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1));

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "selected text");
    assert_eq!(service.backend().read_text().unwrap(), Some("old clipboard".to_string()));
}

#[test]
fn capture_returns_empty_when_copy_produces_no_text() {
    let fake = FakeClipboard::default();
    *fake.current.borrow_mut() = Some("old clipboard".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1));

    let err = service.capture_selected_text().unwrap_err();

    assert_eq!(err.kind, CaptureErrorKind::NoText);
    assert_eq!(service.backend().read_text().unwrap(), Some("old clipboard".to_string()));
}
```

- [x] **Step 2: Run tests and confirm failure**

Run:

```powershell
cargo test --test capture_tests
```

Expected: compile fails because capture types are missing.

- [x] **Step 3: Implement capture core with backend trait**

Modify `src/capture.rs`:

```rust
use crate::error::{AppError, Result};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedText {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureErrorKind {
    NoText,
    ClipboardUnavailable,
    CopyFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureError {
    pub kind: CaptureErrorKind,
    pub message: String,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CaptureError {}

pub trait ClipboardBackend {
    fn read_text(&self) -> Result<Option<String>>;
    fn write_text(&self, text: &str) -> Result<()>;
    fn send_copy(&self) -> Result<()>;
}

pub struct CaptureService<B> {
    backend: B,
    copy_wait: Duration,
}

impl<B: ClipboardBackend> CaptureService<B> {
    pub fn new(backend: B, copy_wait: Duration) -> Self {
        Self { backend, copy_wait }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn capture_selected_text(&self) -> std::result::Result<CapturedText, CaptureError> {
        let previous = self.backend.read_text().map_err(to_capture_error)?;
        self.backend.send_copy().map_err(|err| CaptureError {
            kind: CaptureErrorKind::CopyFailed,
            message: err.to_string(),
        })?;
        thread::sleep(self.copy_wait);

        let copied = self.backend.read_text().map_err(to_capture_error)?;
        if let Some(old) = previous {
            let _ = self.backend.write_text(&old);
        }

        let text = copied.unwrap_or_default();
        if text.trim().is_empty() {
            return Err(CaptureError {
                kind: CaptureErrorKind::NoText,
                message: "没有取到选中文本".to_string(),
            });
        }

        Ok(CapturedText { text })
    }
}

fn to_capture_error(err: AppError) -> CaptureError {
    CaptureError {
        kind: CaptureErrorKind::ClipboardUnavailable,
        message: err.to_string(),
    }
}
```

- [x] **Step 4: Add Windows clipboard backend skeleton**

Append to `src/capture.rs`:

```rust
#[cfg(windows)]
pub struct WindowsClipboardBackend;

#[cfg(windows)]
impl ClipboardBackend for WindowsClipboardBackend {
    fn read_text(&self) -> Result<Option<String>> {
        use windows::Win32::System::DataExchange::{
            CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, CF_UNICODETEXT,
        };
        use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

        unsafe {
            if !IsClipboardFormatAvailable(CF_UNICODETEXT).as_bool() {
                return Ok(None);
            }
            OpenClipboard(None).map_err(|err| AppError::Capture(format!("打开剪贴板失败: {err}")))?;
            let handle = GetClipboardData(CF_UNICODETEXT)
                .map_err(|err| {
                    let _ = CloseClipboard();
                    AppError::Capture(format!("读取剪贴板失败: {err}"))
                })?;
            let ptr = GlobalLock(handle);
            if ptr.is_null() {
                let _ = CloseClipboard();
                return Err(AppError::Capture("锁定剪贴板数据失败".to_string()));
            }
            let mut len = 0usize;
            let wide = ptr as *const u16;
            while *wide.add(len) != 0 {
                len += 1;
            }
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(wide, len));
            let _ = GlobalUnlock(handle);
            let _ = CloseClipboard();
            Ok(Some(text))
        }
    }

    fn write_text(&self, text: &str) -> Result<()> {
        use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData, CF_UNICODETEXT};
        use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

        let mut wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        unsafe {
            OpenClipboard(None).map_err(|err| AppError::Capture(format!("打开剪贴板失败: {err}")))?;
            EmptyClipboard().map_err(|err| {
                let _ = CloseClipboard();
                AppError::Capture(format!("清空剪贴板失败: {err}"))
            })?;
            let bytes = wide.len() * std::mem::size_of::<u16>();
            let handle = GlobalAlloc(GMEM_MOVEABLE, bytes)
                .map_err(|err| {
                    let _ = CloseClipboard();
                    AppError::Capture(format!("分配剪贴板内存失败: {err}"))
                })?;
            let ptr = GlobalLock(handle);
            std::ptr::copy_nonoverlapping(wide.as_mut_ptr() as *const u8, ptr as *mut u8, bytes);
            let _ = GlobalUnlock(handle);
            SetClipboardData(CF_UNICODETEXT, handle).map_err(|err| {
                let _ = CloseClipboard();
                AppError::Capture(format!("写入剪贴板失败: {err}"))
            })?;
            let _ = CloseClipboard();
            Ok(())
        }
    }

    fn send_copy(&self) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
            VK_CONTROL,
        };

        unsafe {
            let inputs = [
                key_input(VK_CONTROL, false),
                key_input(VIRTUAL_KEY(b'C' as u16), false),
                key_input(VIRTUAL_KEY(b'C' as u16), true),
                key_input(VK_CONTROL, true),
            ];
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                return Err(AppError::Capture("发送复制快捷键失败".to_string()));
            }
            Ok(())
        }

        unsafe fn key_input(key: VIRTUAL_KEY, key_up: bool) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: key,
                        wScan: 0,
                        dwFlags: if key_up { KEYEVENTF_KEYUP } else { Default::default() },
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        }
    }
}
```

- [x] **Step 5: Run capture tests and compile check**

Run:

```powershell
cargo test --test capture_tests
cargo test
```

Expected: capture tests pass and project compiles on Windows.

- [x] **Step 6: Commit**

```powershell
git add src/capture.rs tests/capture_tests.rs
git commit -m "feat: add clipboard selection capture"
```

## Task 9: Logging And App Commands

**Files:**
- Modify: `src/logging.rs`
- Modify: `src/command.rs`
- Test: optional unit tests in `tests/logging_tests.rs`

- [x] **Step 1: Define commands**

Modify `src/command.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    TranslateSelection,
    TranslateClipboard,
    OpenSettings,
    OpenLogs,
    RetryTranslation,
    CopyTranslation,
    Exit,
}
```

- [x] **Step 2: Implement privacy-safe logging helpers**

Modify `src/logging.rs`:

```rust
use crate::error::{AppError, Result};
use std::path::PathBuf;

pub fn init_logging() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("dev", "aitsu", "ait")
        .ok_or_else(|| AppError::Config("无法定位日志目录".to_string()))?;
    let log_dir = project_dirs.data_local_dir().join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "ait.log");
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false)
        .init();

    Ok(log_dir)
}

pub fn safe_text_len(text: &str) -> usize {
    text.chars().count()
}
```

- [x] **Step 3: Add a small logging test**

Create `tests/logging_tests.rs`:

```rust
#[test]
fn safe_text_len_counts_chars_without_exposing_text() {
    assert_eq!(ait::logging::safe_text_len("hello世界"), 7);
}
```

- [x] **Step 4: Run tests**

Run:

```powershell
cargo test --test logging_tests
cargo test
```

Expected: tests pass.

- [x] **Step 5: Commit**

```powershell
git add src/command.rs src/logging.rs tests/logging_tests.rs
git commit -m "feat: add app commands and logging"
```

## Task 10: Translation Workflow With Mockable Services

**Files:**
- Modify: `src/app.rs`
- Test: `tests/workflow_tests.rs`

- [x] **Step 1: Write workflow test with fake capture and translator**

Create `tests/workflow_tests.rs`:

```rust
use ait::app::{TranslationWorkflow, WorkflowCapture, WorkflowTranslator};
use ait::capture::CapturedText;
use ait::translator::{ProviderKind, TranslationRequest, TranslationResponse};

struct FakeCapture;

impl WorkflowCapture for FakeCapture {
    fn capture(&self) -> ait::error::Result<CapturedText> {
        Ok(CapturedText { text: "hello".to_string() })
    }
}

struct FakeTranslator;

impl WorkflowTranslator for FakeTranslator {
    fn translate_blocking(&self, request: TranslationRequest) -> ait::error::Result<TranslationResponse> {
        assert_eq!(request.text, "hello");
        Ok(TranslationResponse {
            translated_text: "你好".to_string(),
            provider: ProviderKind::GoogleFree,
        })
    }
}

#[test]
fn translate_selection_captures_then_translates() {
    let workflow = TranslationWorkflow::new(FakeCapture, FakeTranslator);

    let result = workflow.translate_selection("zh-CN").unwrap();

    assert_eq!(result.source_text, "hello");
    assert_eq!(result.translated_text, "你好");
    assert_eq!(result.provider, ProviderKind::GoogleFree);
}
```

- [x] **Step 2: Run test and confirm failure**

Run:

```powershell
cargo test --test workflow_tests
```

Expected: compile fails because workflow types are missing.

- [x] **Step 3: Implement workflow seam**

Modify `src/app.rs`:

```rust
use crate::capture::CapturedText;
use crate::error::Result;
use crate::translator::{ProviderKind, TranslationRequest, TranslationResponse};

pub trait WorkflowCapture {
    fn capture(&self) -> Result<CapturedText>;
}

pub trait WorkflowTranslator {
    fn translate_blocking(&self, request: TranslationRequest) -> Result<TranslationResponse>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationWorkflowResult {
    pub source_text: String,
    pub translated_text: String,
    pub provider: ProviderKind,
}

pub struct TranslationWorkflow<C, T> {
    capture: C,
    translator: T,
}

impl<C, T> TranslationWorkflow<C, T>
where
    C: WorkflowCapture,
    T: WorkflowTranslator,
{
    pub fn new(capture: C, translator: T) -> Self {
        Self { capture, translator }
    }

    pub fn translate_selection(&self, target_lang: &str) -> Result<TranslationWorkflowResult> {
        let captured = self.capture.capture()?;
        let response = self.translator.translate_blocking(TranslationRequest {
            text: captured.text.clone(),
            source_lang: "auto".to_string(),
            target_lang: target_lang.to_string(),
        })?;

        Ok(TranslationWorkflowResult {
            source_text: captured.text,
            translated_text: response.translated_text,
            provider: response.provider,
        })
    }
}

pub fn run() -> Result<()> {
    Ok(())
}
```

- [x] **Step 4: Run workflow tests**

Run:

```powershell
cargo test --test workflow_tests
cargo test
```

Expected: tests pass.

- [x] **Step 5: Commit**

```powershell
git add src/app.rs tests/workflow_tests.rs
git commit -m "feat: add translation workflow"
```

## Task 11: Win32 Tray And Hotkey Message Loop

**Files:**
- Modify: `src/app.rs`
- Modify: `src/hotkey.rs`
- Modify: `src/ui/tray.rs`
- Manual test: `docs/manual-test-checklists/windows-mvp.md`

- [ ] **Step 1: Add hotkey registration wrapper**

Append to `src/hotkey.rs`:

```rust
#[cfg(windows)]
pub struct RegisteredHotkey {
    id: i32,
}

#[cfg(windows)]
impl RegisteredHotkey {
    pub fn register(id: i32, hotkey: Hotkey) -> Result<Self> {
        use windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;
        unsafe {
            RegisterHotKey(None, id, hotkey.win32_modifiers(), hotkey.win32_vk())
                .map_err(|err| AppError::Hotkey(format!("注册快捷键失败: {err}")))?;
        }
        Ok(Self { id })
    }
}

#[cfg(windows)]
impl Drop for RegisteredHotkey {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey(None, self.id);
        }
    }
}
```

- [ ] **Step 2: Implement minimal tray placeholder**

Modify `src/ui/tray.rs` with a Windows-only placeholder that can be improved in the UI task:

```rust
use crate::error::Result;

#[cfg(windows)]
pub struct TrayIcon;

#[cfg(windows)]
impl TrayIcon {
    pub fn create() -> Result<Self> {
        tracing::info!("tray icon placeholder created");
        Ok(Self)
    }
}

#[cfg(not(windows))]
pub struct TrayIcon;

#[cfg(not(windows))]
impl TrayIcon {
    pub fn create() -> Result<Self> {
        Ok(Self)
    }
}
```

- [ ] **Step 3: Implement Win32 message loop with `WM_HOTKEY`**

Modify `src/app.rs` so `run()` calls a Windows implementation:

```rust
pub fn run() -> Result<()> {
    crate::logging::init_logging()?;
    run_platform()
}

#[cfg(not(windows))]
fn run_platform() -> Result<()> {
    tracing::warn!("ait MVP currently supports Windows only");
    Ok(())
}

#[cfg(windows)]
fn run_platform() -> Result<()> {
    use crate::config::{SettingsStore, AppSettings};
    use crate::hotkey::{Hotkey, RegisteredHotkey};
    use crate::ui::tray::TrayIcon;
    use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, TranslateMessage, DispatchMessageW, MSG, WM_HOTKEY};

    let settings_dir = SettingsStore::default_dir()?;
    let settings = SettingsStore::new(settings_dir).load().unwrap_or_else(|_| AppSettings::default());
    let hotkey = settings.hotkey.parse::<Hotkey>()?;
    let _tray = TrayIcon::create()?;
    let _registered = RegisteredHotkey::register(1, hotkey)?;

    tracing::info!("registered hotkey {}", hotkey);

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                tracing::info!("TranslateSelection requested");
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Compile and run manually**

Run:

```powershell
cargo build
cargo run
```

Expected: app stays running. Press `Ctrl+Alt+E`; log records `TranslateSelection requested`.

- [ ] **Step 5: Add manual checklist entry**

Create `docs/manual-test-checklists/windows-mvp.md`:

```markdown
# Windows MVP 手工验证清单

## 快捷键与托盘

- [ ] 启动 `cargo run` 后进程保持运行。
- [ ] 默认快捷键 `Ctrl+Alt+E` 可触发日志 `TranslateSelection requested`。
- [ ] 快捷键被其他程序占用时，应用记录清晰错误。
- [ ] 退出进程后快捷键释放。
```

- [ ] **Step 6: Commit**

```powershell
git add src/app.rs src/hotkey.rs src/ui/tray.rs docs/manual-test-checklists/windows-mvp.md
git commit -m "feat: add win32 hotkey loop"
```

## Task 12: Native Translation Window MVP

**Files:**
- Modify: `src/ui/translate_window.rs`
- Modify: `src/app.rs`
- Manual test: `docs/manual-test-checklists/windows-mvp.md`

- [ ] **Step 1: Implement translation window interface**

Modify `src/ui/translate_window.rs`:

```rust
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct TranslationWindowState {
    pub source_text: String,
    pub translated_text: String,
    pub loading: bool,
    pub error: Option<String>,
}

#[cfg(windows)]
pub struct TranslationWindow {
    state: TranslationWindowState,
}

#[cfg(windows)]
impl TranslationWindow {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: TranslationWindowState {
                source_text: String::new(),
                translated_text: String::new(),
                loading: false,
                error: None,
            },
        })
    }

    pub fn show_loading(&mut self, source_text: String) -> Result<()> {
        self.state.source_text = source_text;
        self.state.translated_text.clear();
        self.state.loading = true;
        self.state.error = None;
        tracing::info!("show translation window loading state");
        Ok(())
    }

    pub fn show_result(&mut self, translated_text: String) -> Result<()> {
        self.state.translated_text = translated_text;
        self.state.loading = false;
        self.state.error = None;
        tracing::info!("show translation window result");
        Ok(())
    }

    pub fn show_error(&mut self, message: String) -> Result<()> {
        self.state.loading = false;
        self.state.error = Some(message);
        tracing::info!("show translation window error");
        Ok(())
    }
}
```

This is a seam first. Replace internals with actual Win32 window creation after workflow integration compiles.

- [ ] **Step 2: Wire hotkey to placeholder window**

Modify Windows `WM_HOTKEY` branch in `src/app.rs`:

```rust
if msg.message == WM_HOTKEY {
    tracing::info!("TranslateSelection requested");
}
```

Replace with:

```rust
if msg.message == WM_HOTKEY {
    tracing::info!("TranslateSelection requested");
    // Next task wires real capture and translation. This keeps the message path visible.
}
```

- [ ] **Step 3: Compile**

Run:

```powershell
cargo build
cargo test
```

Expected: build and tests pass.

- [ ] **Step 4: Extend manual checklist**

Append:

```markdown
## 翻译窗口

- [ ] 快捷键触发后，后续实现应复用同一个翻译窗口而不是创建多个窗口。
- [ ] 窗口默认保持显示，不因失焦自动隐藏。
- [ ] 关闭窗口后，请求结果不能写回已关闭窗口。
```

- [ ] **Step 5: Commit**

```powershell
git add src/ui/translate_window.rs src/app.rs docs/manual-test-checklists/windows-mvp.md
git commit -m "feat: add translation window seam"
```

## Task 13: Wire Capture + Google Translation Into Hotkey Flow

**Files:**
- Modify: `src/app.rs`
- Modify: `src/translator/google_free.rs`
- Modify: `docs/manual-test-checklists/windows-mvp.md`

- [ ] **Step 1: Create blocking wrapper for async translator**

Append to `src/translator/mod.rs`:

```rust
pub fn translate_blocking<T: Translator>(
    translator: &T,
    request: TranslationRequest,
) -> crate::error::Result<TranslationResponse> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|err| crate::error::AppError::Translate(format!("启动翻译运行时失败: {err}")))?;
    runtime.block_on(translator.translate(request))
}
```

- [ ] **Step 2: Implement workflow adapters for Windows capture and Google**

Append to `src/app.rs`:

```rust
#[cfg(windows)]
struct WindowsWorkflowCapture {
    wait_ms: u64,
}

#[cfg(windows)]
impl WorkflowCapture for WindowsWorkflowCapture {
    fn capture(&self) -> Result<crate::capture::CapturedText> {
        let service = crate::capture::CaptureService::new(
            crate::capture::WindowsClipboardBackend,
            std::time::Duration::from_millis(self.wait_ms),
        );
        service.capture_selected_text().map_err(|err| crate::error::AppError::Capture(err.to_string()))
    }
}

#[cfg(windows)]
struct BlockingGoogleTranslator(crate::translator::google_free::GoogleFreeTranslator);

#[cfg(windows)]
impl WorkflowTranslator for BlockingGoogleTranslator {
    fn translate_blocking(&self, request: crate::translator::TranslationRequest) -> Result<crate::translator::TranslationResponse> {
        crate::translator::translate_blocking(&self.0, request)
    }
}
```

- [ ] **Step 3: Call workflow on hotkey**

In `run_platform()`, initialize before message loop:

```rust
let workflow = TranslationWorkflow::new(
    WindowsWorkflowCapture { wait_ms: settings.clipboard_capture.copy_wait_ms },
    BlockingGoogleTranslator(crate::translator::google_free::GoogleFreeTranslator::new()),
);
```

In `WM_HOTKEY` branch:

```rust
match workflow.translate_selection(&settings.target_language) {
    Ok(result) => {
        tracing::info!(
            provider = result.provider.as_log_name(),
            source_len = result.source_text.chars().count(),
            translated_len = result.translated_text.chars().count(),
            "translation completed"
        );
    }
    Err(err) => {
        tracing::warn!(error = %err, "translation failed");
    }
}
```

- [ ] **Step 4: Compile and test manually**

Run:

```powershell
cargo build
cargo test
cargo run
```

Manual expected:

- Select `hello` in Notepad.
- Press `Ctrl+Alt+E`.
- Log records provider `google_free`, source length 5, translated length greater than 0.
- Clipboard text returns to previous text value if it was text.

- [ ] **Step 5: Extend checklist**

Append:

```markdown
## 剪贴板取词与内置 Google 翻译

- [ ] Notepad 选中 `hello` 后按 `Ctrl+Alt+E`，日志显示翻译成功。
- [ ] 浏览器网页中选中文本后按 `Ctrl+Alt+E`，日志显示翻译成功或可理解错误。
- [ ] 剪贴板原本是文本时，翻译后文本剪贴板被恢复。
- [ ] 网络不可用时，日志显示网络错误，不记录完整原文。
```

- [ ] **Step 6: Commit**

```powershell
git add src/app.rs src/translator/mod.rs docs/manual-test-checklists/windows-mvp.md
git commit -m "feat: wire google translation hotkey flow"
```

## Task 14: Minimal Settings Window And Provider Switching

**Files:**
- Modify: `src/ui/settings_window.rs`
- Modify: `src/app.rs`
- Modify: `src/config.rs`
- Manual test: `docs/manual-test-checklists/windows-mvp.md`

- [ ] **Step 1: Add settings view model**

Modify `src/ui/settings_window.rs`:

```rust
use crate::config::{AppSettings, ProviderKind};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsViewModel {
    pub default_provider: ProviderKind,
    pub hotkey: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub has_openai_key: bool,
    pub clipboard_capture_enabled: bool,
    pub copy_wait_ms: u64,
}

impl From<&AppSettings> for SettingsViewModel {
    fn from(settings: &AppSettings) -> Self {
        Self {
            default_provider: settings.default_provider,
            hotkey: settings.hotkey.clone(),
            openai_base_url: settings.openai.base_url.clone(),
            openai_model: settings.openai.model.clone(),
            has_openai_key: settings.openai.encrypted_api_key.is_some(),
            clipboard_capture_enabled: settings.clipboard_capture.enabled,
            copy_wait_ms: settings.clipboard_capture.copy_wait_ms,
        }
    }
}

#[cfg(windows)]
pub struct SettingsWindow;

#[cfg(windows)]
impl SettingsWindow {
    pub fn open(settings: &AppSettings) -> Result<()> {
        let view_model = SettingsViewModel::from(settings);
        tracing::info!(?view_model, "open settings window placeholder");
        Ok(())
    }
}
```

- [ ] **Step 2: Add test for view model**

Create `tests/settings_window_tests.rs`:

```rust
use ait::config::AppSettings;
use ait::ui::settings_window::SettingsViewModel;

#[test]
fn settings_view_model_hides_api_key_value() {
    let mut settings = AppSettings::default();
    settings.openai.encrypted_api_key = Some("encrypted-secret".to_string());

    let vm = SettingsViewModel::from(&settings);

    assert!(vm.has_openai_key);
    assert!(!format!("{vm:?}").contains("encrypted-secret"));
}
```

- [ ] **Step 3: Run tests**

Run:

```powershell
cargo test --test settings_window_tests
cargo test
```

Expected: tests pass.

- [ ] **Step 4: Wire tray/settings command placeholder**

In `src/app.rs`, ensure `OpenSettings` command can call:

```rust
crate::ui::settings_window::SettingsWindow::open(&settings)?;
```

If the full tray menu is not implemented yet, keep this as a callable function from the message loop or a temporary debug path.

- [ ] **Step 5: Extend checklist**

Append:

```markdown
## 设置

- [ ] 设置窗口显示默认翻译提供方。
- [ ] Google 非官方免 Key 翻译不显示 API Key 输入框。
- [ ] OpenAI 兼容接口配置包含 Base URL、Model、API Key、超时。
- [ ] 设置界面提示 Google 非官方免 Key 翻译不是官方 Google Cloud Translation API。
```

- [ ] **Step 6: Commit**

```powershell
git add src/ui/settings_window.rs src/app.rs tests/settings_window_tests.rs docs/manual-test-checklists/windows-mvp.md
git commit -m "feat: add settings window model"
```

## Task 15: OpenAI Provider Switching And DPAPI Key Use

**Files:**
- Modify: `src/app.rs`
- Modify: `src/config.rs`
- Modify: `src/secret.rs`
- Test: add to `tests/config_tests.rs`

- [ ] **Step 1: Add provider selection test**

Append to `tests/config_tests.rs`:

```rust
use ait::config::ProviderKind;

#[test]
fn can_select_openai_compatible_provider() {
    let mut settings = AppSettings::default();
    settings.default_provider = ProviderKind::OpenAiCompatible;
    settings.openai.encrypted_api_key = Some("encrypted".to_string());

    assert_eq!(settings.default_provider, ProviderKind::OpenAiCompatible);
    assert!(settings.openai.encrypted_api_key.is_some());
}
```

- [ ] **Step 2: Add app provider factory**

Append to `src/app.rs`:

```rust
#[cfg(windows)]
fn build_workflow_translator(
    settings: &crate::config::AppSettings,
) -> Result<Box<dyn WorkflowTranslator>> {
    match settings.default_provider {
        crate::config::ProviderKind::GoogleFree => {
            Ok(Box::new(BlockingGoogleTranslator(crate::translator::google_free::GoogleFreeTranslator::new())))
        }
        crate::config::ProviderKind::OpenAiCompatible => {
            let encrypted = settings.openai.encrypted_api_key.as_ref()
                .ok_or_else(|| crate::error::AppError::Translate("API Key 缺失".to_string()))?;
            let api_key = crate::secret::SecretStore::new("ait-openai-api-key").unprotect(encrypted)?;
            let translator = crate::translator::openai_compatible::OpenAiCompatibleTranslator::new(
                crate::translator::openai_compatible::OpenAiCompatibleConfig {
                    base_url: settings.openai.base_url.clone(),
                    api_key,
                    model: settings.openai.model.clone(),
                    timeout_secs: settings.openai.timeout_secs,
                }
            )?;
            Ok(Box::new(BlockingOpenAiTranslator(translator)))
        }
    }
}

#[cfg(windows)]
struct BlockingOpenAiTranslator(crate::translator::openai_compatible::OpenAiCompatibleTranslator);

#[cfg(windows)]
impl WorkflowTranslator for BlockingOpenAiTranslator {
    fn translate_blocking(&self, request: crate::translator::TranslationRequest) -> Result<crate::translator::TranslationResponse> {
        crate::translator::translate_blocking(&self.0, request)
    }
}
```

If `Box<dyn WorkflowTranslator>` needs object safety, keep `WorkflowTranslator` as currently defined with no generics and no async methods.

- [ ] **Step 3: Use provider factory in `run_platform()`**

Replace direct `BlockingGoogleTranslator` construction with `build_workflow_translator(&settings)?`.

- [ ] **Step 4: Run tests and compile**

Run:

```powershell
cargo test
cargo build
```

Expected: all tests pass and build succeeds.

- [ ] **Step 5: Commit**

```powershell
git add src/app.rs src/config.rs src/secret.rs tests/config_tests.rs
git commit -m "feat: switch translation providers"
```

## Task 16: Real Native UI Pass

**Files:**
- Modify: `src/ui/tray.rs`
- Modify: `src/ui/translate_window.rs`
- Modify: `src/ui/settings_window.rs`
- Modify: `src/app.rs`
- Manual test: `docs/manual-test-checklists/windows-mvp.md`

- [ ] **Step 1: Replace tray placeholder with `Shell_NotifyIconW`**

Implement in `src/ui/tray.rs`:

- hidden/message window handle association,
- icon creation or fallback application icon,
- right-click menu with `翻译剪贴板/选区`, `设置`, `查看日志`, `退出`,
- callback message forwarded to `app`.

Use these Win32 APIs:

```text
Shell_NotifyIconW
NOTIFYICONDATAW
CreatePopupMenu
AppendMenuW
TrackPopupMenu
DestroyMenu
```

Run after implementation:

```powershell
cargo build
```

Expected: build succeeds.

- [ ] **Step 2: Replace translation window placeholder with a native window**

Implement in `src/ui/translate_window.rs`:

- registered window class,
- top-level resizable window centered on current screen,
- read-only multiline Edit/RichEdit-style controls for source and translation text,
- buttons: `复制译文`, `重试`, `设置`, `关闭`,
- no Markdown rendering,
- window remains visible after focus loss.

Use plain Win32 controls first. Do not introduce WebView.

Run:

```powershell
cargo build
```

Expected: build succeeds.

- [ ] **Step 3: Replace settings placeholder with minimal native settings window**

Implement in `src/ui/settings_window.rs`:

- provider selector,
- OpenAI Base URL, Model, API Key, Timeout fields,
- hotkey field or fixed text input,
- clipboard capture checkbox,
- explanatory text for Google unofficial no-key provider,
- save/cancel buttons.

On save:

- protect API Key with `SecretStore`,
- save config with `SettingsStore`,
- do not log API Key.

Run:

```powershell
cargo build
```

Expected: build succeeds.

- [ ] **Step 4: Wire UI buttons to app commands**

Modify `src/app.rs` so:

- tray `设置` opens settings,
- tray `退出` exits message loop,
- translation window `重试` repeats last source text,
- translation window `复制译文` writes translated text to clipboard,
- translation window `关闭` hides/closes the window without crashing pending requests.

Run:

```powershell
cargo build
```

Expected: build succeeds.

- [ ] **Step 5: Manual verification**

Run:

```powershell
cargo run
```

Verify:

- tray icon appears,
- tray menu opens,
- settings window opens and saves,
- translation window appears on hotkey,
- translated text is selectable/copyable,
- closing the window does not terminate the tray app,
- app exits from tray menu.

- [ ] **Step 6: Update checklist**

Append:

```markdown
## 原生 UI

- [ ] 托盘图标常驻，右键菜单可用。
- [ ] 设置窗口可保存默认提供方、OpenAI 配置、快捷键和剪贴板取词选项。
- [ ] API Key 保存后不明文出现在 `settings.json`。
- [ ] 翻译窗口可调整大小、可关闭、可复制译文。
- [ ] 翻译窗口失焦后保持显示。
```

- [ ] **Step 7: Commit**

```powershell
git add src/ui src/app.rs docs/manual-test-checklists/windows-mvp.md
git commit -m "feat: add native tray and windows"
```

## Task 17: End-To-End Polish, Error Handling, And Documentation

**Files:**
- Modify: `src/app.rs`
- Modify: `src/error.rs`
- Modify: `src/logging.rs`
- Modify: `README.md`
- Modify: `docs/manual-test-checklists/windows-mvp.md`

- [ ] **Step 1: Add user-facing error summaries**

Modify `src/error.rs` to add:

```rust
impl AppError {
    pub fn user_summary(&self) -> String {
        match self {
            AppError::Hotkey(_) => "快捷键注册失败，请在设置中更换快捷键。".to_string(),
            AppError::Capture(_) => "没有取到选中文本，可以手动粘贴文本后重试。".to_string(),
            AppError::Translate(msg) => format!("翻译失败：{msg}"),
            AppError::Network(_) => "网络连接失败，请检查网络后重试。".to_string(),
            AppError::Secret(_) => "API Key 读取失败，请重新保存接口配置。".to_string(),
            AppError::Config(_) => "配置读取失败，已尝试恢复默认配置。".to_string(),
            AppError::Windows(_) | AppError::Io(_) | AppError::Json(_) => self.to_string(),
        }
    }
}
```

- [ ] **Step 2: Ensure logs are privacy-safe**

Search:

```powershell
rg "source_text|translated_text|api_key|API Key|request.text|captured.text" src tests
```

Expected: no log statement writes raw source text, translated text, or API key. If a log does, replace it with length/provider/status metadata.

- [ ] **Step 3: Add README**

Create `README.md`:

```markdown
# ait

Windows-only lightweight selection translator.

## MVP Behavior

- Tray app, no main window.
- Default hotkey: `Ctrl+Alt+E`.
- Text capture uses clipboard copy and only promises to restore text clipboard content.
- Default translation provider is an unofficial no-key Google Translate endpoint.
- OpenAI-compatible APIs can be configured as an optional provider.
- API keys are protected with Windows DPAPI.

## Build

```powershell
cargo build
```

## Run

```powershell
cargo run
```

## Tests

```powershell
cargo test
```

## Important Limitations

- No UI Automation capture in MVP.
- No OCR in MVP.
- No history in MVP.
- No streaming output in MVP.
- Built-in Google no-key translation is not Google Cloud Translation and may break or be rate-limited.
```
```

- [ ] **Step 4: Complete manual checklist**

Run the whole checklist in `docs/manual-test-checklists/windows-mvp.md` on Windows 10+.

Expected: all MVP items pass or have a documented issue with exact reproduction steps.

- [ ] **Step 5: Final verification commands**

Run:

```powershell
cargo fmt --check
cargo test
cargo build --release
git status --short --ignored
```

Expected:

- formatting passes,
- tests pass,
- release build succeeds,
- no unintended untracked files except ignored `.superpowers/`.

- [ ] **Step 6: Commit**

```powershell
git add README.md src docs/manual-test-checklists/windows-mvp.md
git commit -m "docs: document windows mvp"
```

## Self-Review

- Spec coverage:
  - Tray app: Tasks 11, 16.
  - Fixed hotkey: Tasks 3, 11.
  - Clipboard capture and text restore: Task 8, Task 13.
  - Translation window: Tasks 12, 16.
  - Built-in Google no-key provider: Task 6, Task 13.
  - OpenAI-compatible provider: Task 7, Task 15.
  - Settings window: Task 14, Task 16.
  - DPAPI API Key storage: Task 4, Task 15, Task 16.
  - Logging privacy: Task 9, Task 17.
  - Manual Windows verification: Tasks 11, 13, 16, 17.
- Scope check: UI Automation, OCR, history, streaming, official Google Cloud Translation, Anthropic/Gemini, and WebView are intentionally excluded.
- Placeholder scan: no unresolved placeholder markers or vague "handle edge cases" steps remain.
- Type consistency: shared names are `ProviderKind`, `TranslationRequest`, `TranslationResponse`, `Translator`, `CaptureService`, `ClipboardBackend`, `SettingsStore`, and `SecretStore` throughout the plan.
