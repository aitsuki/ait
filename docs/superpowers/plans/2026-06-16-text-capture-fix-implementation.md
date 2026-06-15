# Text Capture Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not use superpowers:subagent-driven-development or superpowers:using-git-worktrees in this repository.

**Goal:** Fix the core selected-text capture path by making UIA failures fallback cleanly and making clipboard copy follow the open-source reference behavior from Pot Selection.

**Architecture:** Keep `capture` as the boundary for text capture. Split copy triggering from clipboard storage behind traits so tests can verify keyboard release order and clipboard sequence-number handling without Win32. Windows code stays in `src/capture.rs`; `app` continues to only assemble the Windows backends.

**Tech Stack:** Rust stable, `windows-rs`, Win32 clipboard APIs, Win32 `SendInput`, Rust integration tests in `tests/capture_tests.rs`, `cargo test`.

---

## File Structure

- Modify: `src/capture.rs`
  - Extend `ClipboardBackend` with clipboard sequence-number support.
  - Add a `CopyBackend` trait that releases interfering keys and sends clean `Ctrl+C`.
  - Make `CaptureService` generic over clipboard, selection, and copy backends.
  - Treat UIA errors as fallbackable and log strategy outcomes without source text.
  - Implement Windows sequence-number reading with `GetClipboardSequenceNumber`.
  - Move Windows `SendInput` copy behavior into `WindowsCopyBackend`.
- Modify: `src/app.rs`
  - Assemble `CaptureService` with `WindowsClipboardBackend`, `WindowsSelectionBackend`, and `WindowsCopyBackend`.
- Modify: `tests/capture_tests.rs`
  - Update fake clipboard for sequence numbers.
  - Add tests for UIA fallback, key release order, sequence-number timeout, sequence-number success, restore behavior, and UIA errors.
- Modify: `docs/issues/2026-06-16-mvp-validation.md`
  - Check off “无法取词” after tests and manual verification are complete.

---

### Task 1: Add Test Doubles For Clipboard Sequence And Copy Actions

**Files:**
- Modify: `tests/capture_tests.rs`
- Modify: `src/capture.rs`

- [ ] **Step 1: Write failing test-side code that expects new capture traits**

In `tests/capture_tests.rs`, change the import and fake clipboard definitions near the top to this complete block:

```rust
use ait::capture::{
    CaptureErrorKind, CaptureService, ClipboardBackend, CopyAction, CopyBackend, SelectionBackend,
};
use std::cell::RefCell;
use std::time::Duration;

#[derive(Default)]
struct FakeClipboard {
    current: RefCell<Option<String>>,
    copied: RefCell<Option<String>>,
    read_failures: RefCell<usize>,
    writes: RefCell<Vec<String>>,
    sequence: RefCell<u32>,
    advance_sequence_on_copy: RefCell<bool>,
}

impl ClipboardBackend for FakeClipboard {
    fn read_text(&self) -> ait::error::Result<Option<String>> {
        let mut failures = self.read_failures.borrow_mut();
        if *failures > 0 {
            *failures -= 1;
            return Err(ait::error::AppError::Capture(
                "clipboard temporarily unavailable".to_string(),
            ));
        }
        Ok(self.current.borrow().clone())
    }

    fn write_text(&self, text: &str) -> ait::error::Result<()> {
        self.writes.borrow_mut().push(text.to_string());
        *self.current.borrow_mut() = Some(text.to_string());
        *self.sequence.borrow_mut() += 1;
        Ok(())
    }

    fn sequence_number(&self) -> ait::error::Result<u32> {
        Ok(*self.sequence.borrow())
    }
}

#[derive(Default)]
struct FakeCopy {
    actions: RefCell<Vec<CopyAction>>,
}

impl CopyBackend for FakeCopy {
    fn send_copy(&self) -> ait::error::Result<()> {
        self.actions.borrow_mut().extend([
            CopyAction::ReleaseCtrl,
            CopyAction::ReleaseAlt,
            CopyAction::ReleaseShift,
            CopyAction::ReleaseWin,
            CopyAction::ReleaseTab,
            CopyAction::ReleaseEscape,
            CopyAction::ReleaseCapsLock,
            CopyAction::ReleaseC,
            CopyAction::PressCtrl,
            CopyAction::PressC,
            CopyAction::ReleaseC,
            CopyAction::ReleaseCtrl,
        ]);
        Ok(())
    }
}

#[derive(Default)]
struct FakeSelection {
    selected: RefCell<Option<String>>,
    failure: RefCell<Option<ait::error::AppError>>,
}

impl SelectionBackend for FakeSelection {
    fn read_selected_text(&self) -> ait::error::Result<Option<String>> {
        if let Some(err) = self.failure.borrow_mut().take() {
            return Err(err);
        }
        Ok(self.selected.borrow().clone())
    }
}
```

- [ ] **Step 2: Run test compile to verify it fails because production traits do not exist**

Run:

```bash
cargo test --test capture_tests
```

Expected: compile failure mentioning unresolved imports such as `CopyAction` or `CopyBackend`, or missing `sequence_number` in `ClipboardBackend`.

- [ ] **Step 3: Add minimal trait/type definitions to `src/capture.rs`**

In `src/capture.rs`, replace the current `ClipboardBackend` trait with:

```rust
pub trait ClipboardBackend {
    fn read_text(&self) -> Result<Option<String>>;
    fn write_text(&self, text: &str) -> Result<()>;
    fn sequence_number(&self) -> Result<u32>;
}
```

Add this after `SelectionBackend`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyAction {
    ReleaseCtrl,
    ReleaseAlt,
    ReleaseShift,
    ReleaseWin,
    ReleaseTab,
    ReleaseEscape,
    ReleaseCapsLock,
    ReleaseC,
    PressCtrl,
    PressC,
}

pub trait CopyBackend {
    fn send_copy(&self) -> Result<()>;
}

pub struct KeyboardCopyBackend;
```

Temporarily implement `CopyBackend` for `KeyboardCopyBackend` using the old no-op shape so compilation can proceed:

```rust
impl CopyBackend for KeyboardCopyBackend {
    fn send_copy(&self) -> Result<()> {
        Ok(())
    }
}
```

- [ ] **Step 4: Run compile again and observe remaining errors**

Run:

```bash
cargo test --test capture_tests
```

Expected: compile failures in existing tests and Windows implementation because `CaptureService::new` still expects only a clipboard backend and `WindowsClipboardBackend` lacks `sequence_number`.

- [ ] **Step 5: Commit compile-only trait introduction after it builds in later tasks**

Do not commit yet. This task intentionally leaves the tree failing until Task 2 wires the new constructor.

---

### Task 2: Wire CopyBackend Into CaptureService

**Files:**
- Modify: `src/capture.rs`
- Modify: `tests/capture_tests.rs`

- [ ] **Step 1: Update `CaptureService` generics and constructors**

In `src/capture.rs`, replace `CaptureService` and its constructor impls with:

```rust
pub struct CaptureService<B, S = NoSelectionBackend, C = KeyboardCopyBackend> {
    backend: B,
    selection: S,
    copy: C,
    copy_wait: Duration,
}

impl<B: ClipboardBackend> CaptureService<B> {
    pub fn new(backend: B, copy_wait: Duration) -> Self {
        Self {
            backend,
            selection: NoSelectionBackend,
            copy: KeyboardCopyBackend,
            copy_wait,
        }
    }
}

impl<B, S, C> CaptureService<B, S, C>
where
    B: ClipboardBackend,
    S: SelectionBackend,
    C: CopyBackend,
{
    pub fn with_selection<NextSelection>(
        self,
        selection: NextSelection,
    ) -> CaptureService<B, NextSelection, C>
    where
        NextSelection: SelectionBackend,
    {
        CaptureService {
            backend: self.backend,
            selection,
            copy: self.copy,
            copy_wait: self.copy_wait,
        }
    }

    pub fn with_copy<NextCopy>(self, copy: NextCopy) -> CaptureService<B, S, NextCopy>
    where
        NextCopy: CopyBackend,
    {
        CaptureService {
            backend: self.backend,
            selection: self.selection,
            copy,
            copy_wait: self.copy_wait,
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn copy_backend(&self) -> &C {
        &self.copy
    }
```

Keep the existing `capture_selected_text`, `read_clipboard_with_retry`, and `wait_for_copied_text` methods inside this impl for now.

- [ ] **Step 2: Replace old copy call**

In `capture_selected_text`, replace:

```rust
self.backend.send_copy().map_err(|err| CaptureError {
```

with:

```rust
self.copy.send_copy().map_err(|err| CaptureError {
```

- [ ] **Step 3: Update existing tests to use `FakeCopy`**

In every existing test in `tests/capture_tests.rs`, construct services with `.with_copy(FakeCopy::default())`.

Example:

```rust
let service = CaptureService::new(fake, Duration::from_millis(1)).with_copy(FakeCopy::default());
```

For the UIA preference test:

```rust
let service = CaptureService::new(fake, Duration::from_millis(1))
    .with_selection(selection)
    .with_copy(FakeCopy::default());
```

- [ ] **Step 4: Temporarily adapt fake copy behavior for old tests**

For tests that expect copied text to appear, add this helper in `tests/capture_tests.rs`:

```rust
fn simulate_copy(fake: &FakeClipboard) {
    if let Some(text) = fake.copied.borrow().clone() {
        *fake.current.borrow_mut() = Some(text);
        if *fake.advance_sequence_on_copy.borrow() {
            *fake.sequence.borrow_mut() += 1;
        }
    }
}
```

Then in tests that need copied text, call `simulate_copy(service.backend())` immediately before `capture_selected_text()` is not possible because capture triggers copy internally. Leave this helper unused for now; Task 3 will move copy simulation into a composed fake.

- [ ] **Step 5: Run test compile**

Run:

```bash
cargo test --test capture_tests
```

Expected: compile errors may remain for fake copy not mutating clipboard and Windows `sequence_number`; Task 3 resolves the fake model and Task 5 resolves Windows.

---

### Task 3: Add Failing Tests For Sequence-Number Copy Semantics

**Files:**
- Modify: `tests/capture_tests.rs`
- Modify: `src/capture.rs`

- [ ] **Step 1: Replace fake design with shared state**

At the top of `tests/capture_tests.rs`, add:

```rust
#[derive(Default)]
struct FakeCaptureState {
    current: RefCell<Option<String>>,
    copied: RefCell<Option<String>>,
    read_failures: RefCell<usize>,
    writes: RefCell<Vec<String>>,
    sequence: RefCell<u32>,
    advance_sequence_on_copy: RefCell<bool>,
    actions: RefCell<Vec<CopyAction>>,
}

#[derive(Clone, Default)]
struct FakeClipboard(std::rc::Rc<FakeCaptureState>);

#[derive(Clone, Default)]
struct FakeCopy(std::rc::Rc<FakeCaptureState>);
```

Change the fake impl bodies to use `self.0` instead of direct fields. In `FakeCopy::send_copy`, after pushing actions, add:

```rust
if let Some(text) = self.0.copied.borrow().clone() {
    *self.0.current.borrow_mut() = Some(text);
    if *self.0.advance_sequence_on_copy.borrow() {
        *self.0.sequence.borrow_mut() += 1;
    }
}
```

Add a helper:

```rust
fn fake_pair() -> (FakeClipboard, FakeCopy) {
    let state = std::rc::Rc::new(FakeCaptureState::default());
    (FakeClipboard(state.clone()), FakeCopy(state))
}
```

- [ ] **Step 2: Update existing tests to use `fake_pair()`**

For tests needing clipboard and copy, use:

```rust
let (fake, copy) = fake_pair();
*fake.0.current.borrow_mut() = Some("old clipboard".to_string());
*fake.0.copied.borrow_mut() = Some("selected text".to_string());
*fake.0.advance_sequence_on_copy.borrow_mut() = true;
let service = CaptureService::new(fake, Duration::from_millis(20)).with_copy(copy);
```

Update assertions from `service.backend().current` to `service.backend().0.current`.

- [ ] **Step 3: Add failing test for released keys**

Append:

```rust
#[test]
fn capture_releases_interfering_keys_before_copy() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some("selected text".to_string());
    *fake.0.advance_sequence_on_copy.borrow_mut() = true;
    let service = CaptureService::new(fake, Duration::from_millis(20)).with_copy(copy.clone());

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "selected text");
    assert_eq!(
        *copy.0.actions.borrow(),
        vec![
            CopyAction::ReleaseCtrl,
            CopyAction::ReleaseAlt,
            CopyAction::ReleaseShift,
            CopyAction::ReleaseWin,
            CopyAction::ReleaseTab,
            CopyAction::ReleaseEscape,
            CopyAction::ReleaseCapsLock,
            CopyAction::ReleaseC,
            CopyAction::PressCtrl,
            CopyAction::PressC,
            CopyAction::ReleaseC,
            CopyAction::ReleaseCtrl,
        ]
    );
}
```

- [ ] **Step 4: Add failing test for unchanged sequence number**

Append:

```rust
#[test]
fn capture_fails_when_copy_does_not_change_clipboard_sequence() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some("selected text".to_string());
    *fake.0.advance_sequence_on_copy.borrow_mut() = false;
    let service = CaptureService::new(fake, Duration::from_millis(1)).with_copy(copy);

    let err = service.capture_selected_text().unwrap_err();

    assert_eq!(err.kind, CaptureErrorKind::CopyFailed);
    assert_eq!(
        service.backend().read_text().unwrap(),
        Some("old clipboard".to_string())
    );
}
```

- [ ] **Step 5: Add failing test for UIA error fallback**

Append:

```rust
#[test]
fn capture_falls_back_to_clipboard_when_selection_backend_errors() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some("selected text".to_string());
    *fake.0.advance_sequence_on_copy.borrow_mut() = true;
    let selection = FakeSelection::default();
    *selection.failure.borrow_mut() = Some(ait::error::AppError::Capture(
        "focus element does not support TextPattern".to_string(),
    ));
    let service = CaptureService::new(fake, Duration::from_millis(20))
        .with_selection(selection)
        .with_copy(copy);

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "selected text");
}
```

- [ ] **Step 6: Run tests to verify RED**

Run:

```bash
cargo test --test capture_tests
```

Expected: at least `capture_fails_when_copy_does_not_change_clipboard_sequence` and `capture_falls_back_to_clipboard_when_selection_backend_errors` fail. The failure should be behavioral, not syntax.

---

### Task 4: Implement Sequence-Number Clipboard Copy In CaptureService

**Files:**
- Modify: `src/capture.rs`
- Modify: `tests/capture_tests.rs`

- [ ] **Step 1: Make UIA errors fallbackable**

In `capture_selected_text`, replace the initial selection block with:

```rust
match self.selection.read_selected_text() {
    Ok(Some(text)) if !text.trim().is_empty() => {
        tracing::debug!(
            strategy = "uia_focused_selection",
            text_len = text.chars().count(),
            "captured selected text"
        );
        return Ok(CapturedText { text });
    }
    Ok(_) => {
        tracing::debug!(
            strategy = "uia_focused_selection",
            "selection backend returned no text"
        );
    }
    Err(err) => {
        tracing::debug!(
            strategy = "uia_focused_selection",
            error = %err,
            "selection backend failed; falling back to clipboard copy"
        );
    }
}
```

- [ ] **Step 2: Replace old clipboard copy flow**

Replace the clipboard portion of `capture_selected_text` with:

```rust
let previous = self.read_clipboard_with_retry()?;
if previous.is_some() {
    self.backend.write_text("").map_err(to_capture_error)?;
}
let sequence_before = self.backend.sequence_number().map_err(to_capture_error)?;
self.copy.send_copy().map_err(|err| CaptureError {
    kind: CaptureErrorKind::CopyFailed,
    message: err.to_string(),
})?;

let sequence_changed = self.wait_for_clipboard_sequence_change(sequence_before)?;
let copied = if sequence_changed {
    self.read_clipboard_with_retry()?
} else {
    None
};
if let Some(old) = previous {
    let _ = self.backend.write_text(&old);
}

if !sequence_changed {
    return Err(CaptureError {
        kind: CaptureErrorKind::CopyFailed,
        message: "复制后剪贴板没有变化".to_string(),
    });
}

let text = copied.unwrap_or_default();
if text.trim().is_empty() {
    return Err(CaptureError {
        kind: CaptureErrorKind::NoText,
        message: "没有取到选中文本".to_string(),
    });
}

tracing::debug!(
    strategy = "clipboard_copy",
    text_len = text.chars().count(),
    "captured selected text"
);
Ok(CapturedText { text })
```

- [ ] **Step 3: Add sequence wait helper**

Replace `wait_for_copied_text` with:

```rust
fn wait_for_clipboard_sequence_change(
    &self,
    previous_sequence: u32,
) -> std::result::Result<bool, CaptureError> {
    let deadline = Instant::now() + self.copy_wait;
    loop {
        match self.backend.sequence_number() {
            Ok(sequence) if sequence != previous_sequence => return Ok(true),
            Ok(_) => {}
            Err(err) => {
                tracing::debug!(
                    error = %err,
                    "clipboard sequence read failed while waiting for copied text"
                );
            }
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(10));
    }
}
```

- [ ] **Step 4: Run capture tests**

Run:

```bash
cargo test --test capture_tests
```

Expected: tests compile and pass except any tests still using old direct fake fields. Fix field references only, without changing assertions.

- [ ] **Step 5: Commit service behavior**

Run:

```bash
git add src/capture.rs tests/capture_tests.rs
git commit -m "fix: use sequence-aware clipboard capture"
```

Expected: commit succeeds.

---

### Task 5: Implement Windows Clipboard Sequence And Clean Copy Backend

**Files:**
- Modify: `src/capture.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add Windows sequence-number support**

In `impl ClipboardBackend for WindowsClipboardBackend`, add:

```rust
fn sequence_number(&self) -> Result<u32> {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

    unsafe { Ok(GetClipboardSequenceNumber()) }
}
```

- [ ] **Step 2: Rename generic keyboard backend to Windows-specific copy backend**

In `src/capture.rs`, replace:

```rust
pub struct KeyboardCopyBackend;

impl CopyBackend for KeyboardCopyBackend {
    fn send_copy(&self) -> Result<()> {
        Ok(())
    }
}
```

with:

```rust
pub struct NoCopyBackend;

impl CopyBackend for NoCopyBackend {
    fn send_copy(&self) -> Result<()> {
        Ok(())
    }
}
```

Update `CaptureService` default generic and constructor from `KeyboardCopyBackend` to `NoCopyBackend`.

- [ ] **Step 3: Add WindowsCopyBackend**

In `src/capture.rs`, before `WindowsSelectionBackend`, add:

```rust
#[cfg(windows)]
pub struct WindowsCopyBackend;

#[cfg(windows)]
impl CopyBackend for WindowsCopyBackend {
    fn send_copy(&self) -> Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
            VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_MENU, VK_SHIFT, VK_TAB,
        };

        unsafe {
            let inputs = [
                key_input(VK_CONTROL, true),
                key_input(VK_MENU, true),
                key_input(VK_SHIFT, true),
                key_input(VK_LWIN, true),
                key_input(VK_TAB, true),
                key_input(VK_ESCAPE, true),
                key_input(VIRTUAL_KEY(0x14), true),
                key_input(VIRTUAL_KEY(b'C' as u16), true),
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
                        dwFlags: if key_up {
                            KEYEVENTF_KEYUP
                        } else {
                            Default::default()
                        },
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        }
    }
}
```

- [ ] **Step 4: Remove old copy method from WindowsClipboardBackend**

Delete the old `fn send_copy(&self) -> Result<()>` method from `impl ClipboardBackend for WindowsClipboardBackend`.

- [ ] **Step 5: Update app assembly**

In `src/app.rs`, update `WindowsWorkflowCapture::capture`:

```rust
let service = crate::capture::CaptureService::new(
    crate::capture::WindowsClipboardBackend,
    std::time::Duration::from_millis(self.wait_ms),
)
.with_selection(crate::capture::WindowsSelectionBackend)
.with_copy(crate::capture::WindowsCopyBackend);
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test --test capture_tests
```

Expected: all capture tests pass.

- [ ] **Step 7: Run full tests**

Run:

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 8: Commit Windows backend**

Run:

```bash
git add src/capture.rs src/app.rs tests/capture_tests.rs
git commit -m "fix: send clean copy shortcut on windows"
```

Expected: commit succeeds.

---

### Task 6: Update MVP Issue And Manual Verification Notes

**Files:**
- Modify: `docs/issues/2026-06-16-mvp-validation.md`

- [ ] **Step 1: Run manual Windows validation**

Run the application normally, then verify these cases with default `Ctrl+Alt+E`:

```text
1. Notepad: type "hello world", select it, press Ctrl+Alt+E.
   Expected: translation window shows "hello world" as source text.

2. Browser input box: type "browser input", select it, press Ctrl+Alt+E.
   Expected: translation window shows "browser input" as source text.

3. Browser page text: select normal selectable page text, press Ctrl+Alt+E.
   Expected: translation window shows selected page text as source text.
```

- [ ] **Step 2: Check logs**

Open the application log and confirm at least one line identifies:

```text
strategy=uia_focused_selection
```

or:

```text
strategy=clipboard_copy
```

Expected: log does not include full selected source text.

- [ ] **Step 3: Update issue checklist**

In `docs/issues/2026-06-16-mvp-validation.md`, change:

```markdown
- [ ] 无法取词
```

to:

```markdown
- [x] 无法取词
```

- [ ] **Step 4: Run final verification**

Run:

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 5: Commit issue update**

Run:

```bash
git add docs/issues/2026-06-16-mvp-validation.md
git commit -m "docs: mark text capture issue fixed"
```

Expected: commit succeeds.

---

## Self-Review

- Spec coverage:
  - UIA fallbackable errors: Task 3 and Task 4.
  - Clean clipboard copy with released modifiers: Task 3 and Task 5.
  - Clipboard sequence-number detection after our own clipboard clear: Task 3, Task 4, Task 5.
  - Logging without full source text: Task 4 and Task 6.
  - ADR 0001 no OCR: no task introduces OCR or OCR dependencies.
  - App boundary preserved: Task 5 only changes assembly in `src/app.rs`.
- Placeholder scan:
  - No `TBD`, `TODO`, or undefined future steps are required.
  - Every code-changing step includes concrete code or exact edits.
- Type consistency:
  - `ClipboardBackend::sequence_number`, `CopyBackend::send_copy`, `CopyAction`, `NoCopyBackend`, and `WindowsCopyBackend` are introduced before use.
  - `CaptureService::with_copy` is used consistently in tests and app assembly.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-16-text-capture-fix-implementation.md`.

Because this repository forbids `superpowers:subagent-driven-development` and `superpowers:using-git-worktrees`, the only allowed execution route for this plan is inline execution with `superpowers:executing-plans`.
