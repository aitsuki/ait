# 翻译窗口操作行为调整 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让翻译窗口只负责当前文本的翻译与展示，托盘菜单只负责显示窗口，窗口内支持更自然的文本选择与 Esc 快速隐藏。

**Architecture:** 把“取词翻译”和“按窗口现有文本翻译”拆成两条路径，前者保留现有 capture 流程，后者直接复用翻译器。托盘菜单只做窗口显示/激活，翻译窗口只处理自己的显示状态和编辑控件交互，不再反向触发取词流程。

**Tech Stack:** Rust, Win32 API, `windows` crate, 现有 `cargo test`/集成测试。

---

### Task 1: 增加按文本翻译路径，确保不再依赖 capture

**Files:**
- Modify: `src/app.rs`
- Modify: `tests/workflow_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
struct PanicCapture;

impl WorkflowCapture for PanicCapture {
    fn capture(&self) -> ait::error::Result<CapturedText> {
        panic!("capture must not run for direct text translation");
    }
}

#[test]
fn translate_text_translates_without_capture() {
    let workflow = TranslationWorkflow::new(PanicCapture, FakeTranslator);

    let result = workflow.translate_text("hello", "zh-CN").unwrap();

    assert_eq!(result.source_text, "hello");
    assert_eq!(result.translated_text, "你好");
    assert_eq!(result.provider, ProviderKind::GoogleFree);
}

#[test]
fn translate_text_rejects_empty_source() {
    let workflow = TranslationWorkflow::new(PanicCapture, FakeTranslator);

    let err = workflow.translate_text("   ", "zh-CN").unwrap_err();

    assert!(err.to_string().contains("原文为空"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test workflow_tests translate_text_translates_without_capture -- --exact
```

Expected: fail because `translate_text` does not exist yet.

- [ ] **Step 3: Write minimal implementation**

```rust
impl<C, T> TranslationWorkflow<C, T>
where
    C: WorkflowCapture,
    T: WorkflowTranslator,
{
    pub fn translate_text(&self, source_text: &str, target_lang: &str) -> Result<TranslationWorkflowResult> {
        self.translate_text_with_observer(source_text, target_lang, &mut ())
    }

    pub fn translate_text_with_observer<O>(
        &self,
        source_text: &str,
        target_lang: &str,
        observer: &mut O,
    ) -> Result<TranslationWorkflowResult>
    where
        O: TranslationObserver,
    {
        let source_text = source_text.trim();
        if source_text.is_empty() {
            return Err(crate::error::AppError::Translate("原文为空".to_string()));
        }

        observer.translation_started()?;
        observer.source_captured(source_text)?;
        let response = self.translator.translate_blocking(TranslationRequest {
            text: source_text.to_string(),
            source_lang: "auto".to_string(),
            target_lang: target_lang.to_string(),
        })?;

        let result = TranslationWorkflowResult {
            source_text: source_text.to_string(),
            translated_text: response.translated_text,
            provider: response.provider,
        };
        observer.translation_succeeded(&result)?;
        Ok(result)
    }
}

pub fn translate_selection_with_observer<O>(
    &self,
    target_lang: &str,
    observer: &mut O,
) -> Result<TranslationWorkflowResult>
where
    O: TranslationObserver,
{
    observer.translation_started()?;
    let captured = self.capture.capture()?;
    self.translate_text_with_observer(&captured.text, target_lang, observer)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test workflow_tests translate_text
```

Expected: both tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs tests/workflow_tests.rs
git commit -m "feat: add direct text translation path"
```

---

### Task 2: 改造托盘菜单为“显示翻译窗口”，只激活窗口不触发翻译

**Files:**
- Modify: `src/ui/tray.rs`
- Modify: `src/app.rs`
- Modify: `tests/workflow_tests.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn tray_show_window_menu_id_maps_to_show_window_action() {
    assert_eq!(
        ait::app::tray_action_from_menu_id(ait::ui::tray::MENU_SHOW_TRANSLATION_WINDOW),
        ait::app::TrayAction::ShowTranslationWindow
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test workflow_tests tray_show_window_menu_id_maps_to_show_window_action -- --exact
```

Expected: fail because `TrayAction` / `tray_action_from_menu_id` do not exist yet.

- [ ] **Step 3: Write minimal implementation**

```rust
#[cfg(windows)]
pub const MENU_SHOW_TRANSLATION_WINDOW: usize = 1001;

#[cfg(windows)]
pub enum TrayAction {
    ShowTranslationWindow,
    OpenSettings,
    OpenLogs,
    Exit,
    Unknown,
}

#[cfg(windows)]
pub fn tray_action_from_menu_id(menu_id: usize) -> TrayAction {
    match menu_id {
        crate::ui::tray::MENU_SHOW_TRANSLATION_WINDOW => TrayAction::ShowTranslationWindow,
        crate::ui::tray::MENU_SETTINGS => TrayAction::OpenSettings,
        crate::ui::tray::MENU_OPEN_LOGS => TrayAction::OpenLogs,
        crate::ui::tray::MENU_EXIT => TrayAction::Exit,
        _ => TrayAction::Unknown,
    }
}

// tray popup handler
match tray_action_from_menu_id(msg.wParam.0) {
    TrayAction::ShowTranslationWindow => {
        let _ = translation_window.show_window();
    }
    TrayAction::OpenSettings => {
        let _ = handle_app_command(crate::command::AppCommand::OpenSettings, &settings);
    }
    TrayAction::OpenLogs => {
        tracing::info!("OpenLogs requested");
    }
    TrayAction::Exit => {
        if handle_app_command(crate::command::AppCommand::Exit, &settings)? {
            PostQuitMessage(0);
        }
    }
    TrayAction::Unknown => {}
}
```

`TranslationWindow::show_window()` should only reveal/activate the existing window and preserve current contents. It must not call `show_starting`, `show_loading`, or `perform_translation`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test workflow_tests tray_show_window_menu_id_maps_to_show_window_action -- --exact
```

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/ui/tray.rs src/app.rs tests/workflow_tests.rs
git commit -m "feat: make tray menu show translation window"
```

---

### Task 3: 重构翻译窗口按钮和输入框交互

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/ui/translate_window.rs`
- Modify: `src/app.rs`
- Modify: `docs/manual-test-checklists/windows-mvp.md`

- [ ] **Step 1: Write the failing test**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditShortcutAction {
    None,
    SelectAll,
    HideWindow,
}

fn edit_shortcut_action(vk: u32, ctrl_down: bool) -> EditShortcutAction {
    const VK_A: u32 = 0x41;
    const VK_ESCAPE: u32 = 0x1B;

    if ctrl_down && vk == VK_A {
        EditShortcutAction::SelectAll
    } else if vk == VK_ESCAPE {
        EditShortcutAction::HideWindow
    } else {
        EditShortcutAction::None
    }
}

#[test]
fn edit_shortcut_action_handles_ctrl_a_and_escape() {
    assert_eq!(edit_shortcut_action(0x41, true), EditShortcutAction::SelectAll);
    assert_eq!(edit_shortcut_action(0x1B, false), EditShortcutAction::HideWindow);
    assert_eq!(edit_shortcut_action(0x42, false), EditShortcutAction::None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cargo test edit_shortcut_action_handles_ctrl_a_and_escape -- --exact
```

Expected: fail because the helper does not exist yet.

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditShortcutAction {
    None,
    SelectAll,
    HideWindow,
}

fn edit_shortcut_action(vk: u32, ctrl_down: bool) -> EditShortcutAction {
    const VK_A: u32 = 0x41;
    const VK_ESCAPE: u32 = 0x1B;

    if ctrl_down && vk == VK_A {
        EditShortcutAction::SelectAll
    } else if vk == VK_ESCAPE {
        EditShortcutAction::HideWindow
    } else {
        EditShortcutAction::None
    }
}

unsafe extern "system" fn edit_subclass_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    _id_subclass: usize,
    _ref_data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::Controls::DefSubclassProc;
    use windows::Win32::UI::WindowsAndMessaging::{
        EM_SETSEL, GetKeyState, PostMessageW, WM_KEYDOWN,
    };

    if msg == WM_KEYDOWN {
        let ctrl_down = unsafe { GetKeyState(windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL.0 as i32) } < 0;
        match edit_shortcut_action(wparam.0 as u32, ctrl_down) {
            EditShortcutAction::SelectAll => {
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                        hwnd,
                        EM_SETSEL,
                        windows::Win32::Foundation::WPARAM(0),
                        windows::Win32::Foundation::LPARAM(-1),
                    );
                }
                return windows::Win32::Foundation::LRESULT(0);
            }
            EditShortcutAction::HideWindow => {
                unsafe {
                    let parent = windows::Win32::UI::WindowsAndMessaging::GetParent(hwnd);
                    let _ = PostMessageW(
                        Some(parent),
                        windows::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                        windows::Win32::Foundation::WPARAM(0),
                        windows::Win32::Foundation::LPARAM(0),
                    );
                }
                return windows::Win32::Foundation::LRESULT(0);
            }
            EditShortcutAction::None => {}
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
```

Then:
- add `Win32_UI_Controls` to the `windows` crate features in `Cargo.toml`
- remove `复制译文`, `重试`, `设置` buttons
- rename the remaining button text to `翻译`
- expose a `TranslationWindow::source_text()` method that reads the current `source_edit` content
- make the `翻译` button post `WM_TRANSLATE_WINDOW_SOURCE`
- handle `WM_TRANSLATE_WINDOW_SOURCE` in `src/app.rs` by reading `translation_window.source_text()` and calling `workflow.translate_text_with_observer(...)`
- keep the native multiline edit control behavior so double-click and triple-click selection remain system-managed
- install the subclass on both edit controls so `Ctrl+A` and `Esc` work consistently

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cargo test edit_shortcut_action_handles_ctrl_a_and_escape -- --exact
```

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/app.rs src/ui/translate_window.rs docs/manual-test-checklists/windows-mvp.md
git commit -m "feat: simplify translation window actions"
```

---

### Task 4: 端到端验证与收尾

**Files:**
- Modify: `docs/manual-test-checklists/windows-mvp.md`（若执行中发现还要补充验证项）

- [ ] **Step 1: Run the focused test suite**

Run:
```bash
cargo test --test workflow_tests
```

Expected: all workflow tests pass, including direct translation and tray command mapping.

- [ ] **Step 2: Run the full test suite**

Run:
```bash
cargo test
```

Expected: pass.

- [ ] **Step 3: Run format check**

Run:
```bash
cargo fmt --check
```

Expected: pass.

- [ ] **Step 4: Manual desktop verification**

Verify these items in the Windows MVP checklist:
- 托盘“显示翻译窗口”只显示并激活窗口，不触发取词或翻译
- 已显示窗口再次点击托盘入口时会带到最顶部并获取焦点
- 翻译窗口底部只剩“翻译”按钮
- `Esc` 可隐藏翻译窗口
- 原文区和译文区支持双击、三连击、`Ctrl+A`
- 点击“翻译”只翻译原文区当前文本，不重新取词

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "test: verify translation window action changes"
```
