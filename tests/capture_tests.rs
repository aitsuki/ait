use ait::capture::{
    CaptureErrorKind, CaptureService, ClipboardBackend, CopyAction, CopyBackend, SelectionBackend,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

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
struct FakeClipboard(Rc<FakeCaptureState>);

#[derive(Clone, Default)]
struct FakeCopy(Rc<FakeCaptureState>);

impl ClipboardBackend for FakeClipboard {
    fn read_text(&self) -> ait::error::Result<Option<String>> {
        let mut failures = self.0.read_failures.borrow_mut();
        if *failures > 0 {
            *failures -= 1;
            return Err(ait::error::AppError::Capture(
                "clipboard temporarily unavailable".to_string(),
            ));
        }
        Ok(self.0.current.borrow().clone())
    }

    fn write_text(&self, text: &str) -> ait::error::Result<()> {
        self.0.writes.borrow_mut().push(text.to_string());
        *self.0.current.borrow_mut() = Some(text.to_string());
        *self.0.sequence.borrow_mut() += 1;
        Ok(())
    }

    fn sequence_number(&self) -> ait::error::Result<u32> {
        Ok(*self.0.sequence.borrow())
    }
}

impl CopyBackend for FakeCopy {
    fn send_copy(&self) -> ait::error::Result<()> {
        self.0.actions.borrow_mut().extend([
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
        if let Some(text) = self.0.copied.borrow().clone() {
            *self.0.current.borrow_mut() = Some(text);
            if *self.0.advance_sequence_on_copy.borrow() {
                *self.0.sequence.borrow_mut() += 1;
            }
        }
        Ok(())
    }
}

fn fake_pair() -> (FakeClipboard, FakeCopy) {
    let state = Rc::new(FakeCaptureState::default());
    (FakeClipboard(state.clone()), FakeCopy(state))
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

#[test]
fn capture_restores_previous_text_clipboard() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some("selected text".to_string());
    *fake.0.advance_sequence_on_copy.borrow_mut() = true;
    let service = CaptureService::new(fake, Duration::from_millis(20)).with_copy(copy);

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "selected text");
    assert_eq!(
        service.backend().read_text().unwrap(),
        Some("old clipboard".to_string())
    );
}

#[test]
fn capture_returns_empty_when_copy_produces_no_text() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some(String::new());
    *fake.0.advance_sequence_on_copy.borrow_mut() = true;
    let service = CaptureService::new(fake, Duration::from_millis(20)).with_copy(copy);

    let err = service.capture_selected_text().unwrap_err();

    assert_eq!(err.kind, CaptureErrorKind::NoText);
    assert_eq!(
        service.backend().read_text().unwrap(),
        Some("old clipboard".to_string())
    );
}

#[test]
fn capture_prefers_selection_backend_without_touching_clipboard() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some("clipboard copy".to_string());
    let selection = FakeSelection::default();
    *selection.selected.borrow_mut() = Some("uia selection".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1))
        .with_selection(selection)
        .with_copy(copy.clone());

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "uia selection");
    assert!(copy.0.actions.borrow().is_empty());
    assert!(service.backend().0.writes.borrow().is_empty());
}

#[test]
fn capture_normalizes_known_uia_text_artifacts() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some("clipboard copy".to_string());
    let selection = FakeSelection::default();
    *selection.selected.borrow_mut() =
        Some("left；right\u{fffc}\u{fffd}“quoted”‘single’".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1))
        .with_selection(selection)
        .with_copy(copy.clone());

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "left;right\"quoted\"'single'");
    assert!(copy.0.actions.borrow().is_empty());
}

#[test]
fn capture_polls_until_copied_text_arrives() {
    #[derive(Default)]
    struct DelayedState {
        reads: RefCell<usize>,
        text: RefCell<Option<String>>,
        sequence: RefCell<u32>,
    }

    #[derive(Clone, Default)]
    struct DelayedClipboard(Rc<DelayedState>);

    #[derive(Clone, Default)]
    struct DelayedCopy(Rc<DelayedState>);

    impl ClipboardBackend for DelayedClipboard {
        fn read_text(&self) -> ait::error::Result<Option<String>> {
            let mut reads = self.0.reads.borrow_mut();
            *reads += 1;
            Ok(self.0.text.borrow().clone())
        }

        fn write_text(&self, text: &str) -> ait::error::Result<()> {
            *self.0.text.borrow_mut() = Some(text.to_string());
            *self.0.sequence.borrow_mut() += 1;
            Ok(())
        }

        fn sequence_number(&self) -> ait::error::Result<u32> {
            Ok(*self.0.sequence.borrow())
        }
    }

    impl CopyBackend for DelayedCopy {
        fn send_copy(&self) -> ait::error::Result<()> {
            *self.0.text.borrow_mut() = Some("delayed selected text".to_string());
            *self.0.sequence.borrow_mut() += 1;
            Ok(())
        }
    }

    let state = Rc::new(DelayedState::default());
    *state.text.borrow_mut() = Some("old clipboard".to_string());
    let service = CaptureService::new(DelayedClipboard(state.clone()), Duration::from_millis(20))
        .with_copy(DelayedCopy(state));

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "delayed selected text");
}

#[test]
fn capture_retries_when_clipboard_is_temporarily_unavailable() {
    let (fake, copy) = fake_pair();
    *fake.0.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.0.copied.borrow_mut() = Some("selected text".to_string());
    *fake.0.advance_sequence_on_copy.borrow_mut() = true;
    *fake.0.read_failures.borrow_mut() = 1;
    let service = CaptureService::new(fake, Duration::from_millis(20)).with_copy(copy);

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "selected text");
}

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
