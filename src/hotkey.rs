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

impl Modifiers {
    pub fn any(self) -> bool {
        self.ctrl || self.alt || self.shift || self.win
    }
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
        let mut modifiers = Modifiers {
            ctrl: false,
            alt: false,
            shift: false,
            win: false,
        };
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
                    key = Some(KeyCode::Char(
                        value.chars().next().unwrap().to_ascii_uppercase(),
                    ));
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
        if !modifiers.any() {
            return Err(AppError::Hotkey(
                "快捷键必须至少包含一个修饰键".to_string(),
            ));
        }
        Ok(Self { modifiers, key })
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.modifiers.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_string());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_string());
        }
        if self.modifiers.win {
            parts.push("Win".to_string());
        }
        parts.push(match self.key {
            KeyCode::Char(ch) => ch.to_string(),
            KeyCode::Function(n) => format!("F{n}"),
        });
        write!(f, "{}", parts.join("+"))
    }
}

#[cfg(windows)]
impl Hotkey {
    pub fn win32_modifiers(self) -> windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
        };
        let mut bits = HOT_KEY_MODIFIERS(0);
        if self.modifiers.ctrl {
            bits |= MOD_CONTROL;
        }
        if self.modifiers.alt {
            bits |= MOD_ALT;
        }
        if self.modifiers.shift {
            bits |= MOD_SHIFT;
        }
        if self.modifiers.win {
            bits |= MOD_WIN;
        }
        bits
    }

    pub fn win32_vk(self) -> u32 {
        match self.key {
            KeyCode::Char(ch) => ch as u32,
            KeyCode::Function(n) => 0x70 + (n as u32) - 1,
        }
    }
}

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
