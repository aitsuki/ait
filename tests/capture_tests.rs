use ait::capture::{CaptureErrorKind, CaptureService, ClipboardBackend, SelectionBackend};
use std::cell::RefCell;
use std::time::Duration;

#[derive(Default)]
struct FakeClipboard {
    current: RefCell<Option<String>>,
    copied: RefCell<Option<String>>,
    read_failures: RefCell<usize>,
    writes: RefCell<Vec<String>>,
    copy_calls: RefCell<usize>,
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
        Ok(())
    }

    fn send_copy(&self) -> ait::error::Result<()> {
        *self.copy_calls.borrow_mut() += 1;
        if let Some(text) = self.copied.borrow().clone() {
            *self.current.borrow_mut() = Some(text);
        }
        Ok(())
    }
}

#[derive(Default)]
struct FakeSelection {
    selected: RefCell<Option<String>>,
}

impl SelectionBackend for FakeSelection {
    fn read_selected_text(&self) -> ait::error::Result<Option<String>> {
        Ok(self.selected.borrow().clone())
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
    assert_eq!(
        service.backend().read_text().unwrap(),
        Some("old clipboard".to_string())
    );
}

#[test]
fn capture_returns_empty_when_copy_produces_no_text() {
    let fake = FakeClipboard::default();
    *fake.current.borrow_mut() = Some("old clipboard".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1));

    let err = service.capture_selected_text().unwrap_err();

    assert_eq!(err.kind, CaptureErrorKind::NoText);
    assert_eq!(
        service.backend().read_text().unwrap(),
        Some("old clipboard".to_string())
    );
}

#[test]
fn capture_prefers_selection_backend_without_touching_clipboard() {
    let fake = FakeClipboard::default();
    *fake.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.copied.borrow_mut() = Some("clipboard copy".to_string());
    let selection = FakeSelection::default();
    *selection.selected.borrow_mut() = Some("uia selection".to_string());
    let service = CaptureService::new(fake, Duration::from_millis(1)).with_selection(selection);

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "uia selection");
    assert_eq!(*service.backend().copy_calls.borrow(), 0);
    assert!(service.backend().writes.borrow().is_empty());
}

#[test]
fn capture_polls_until_copied_text_arrives() {
    #[derive(Default)]
    struct DelayedClipboard {
        reads: RefCell<usize>,
        copy_sent: RefCell<bool>,
    }

    impl ClipboardBackend for DelayedClipboard {
        fn read_text(&self) -> ait::error::Result<Option<String>> {
            let mut reads = self.reads.borrow_mut();
            *reads += 1;
            if *self.copy_sent.borrow() && *reads >= 4 {
                Ok(Some("delayed selected text".to_string()))
            } else {
                Ok(Some(String::new()))
            }
        }

        fn write_text(&self, _text: &str) -> ait::error::Result<()> {
            Ok(())
        }

        fn send_copy(&self) -> ait::error::Result<()> {
            *self.copy_sent.borrow_mut() = true;
            Ok(())
        }
    }

    let service = CaptureService::new(DelayedClipboard::default(), Duration::from_millis(20));

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "delayed selected text");
}

#[test]
fn capture_retries_when_clipboard_is_temporarily_unavailable() {
    let fake = FakeClipboard::default();
    *fake.current.borrow_mut() = Some("old clipboard".to_string());
    *fake.copied.borrow_mut() = Some("selected text".to_string());
    *fake.read_failures.borrow_mut() = 1;
    let service = CaptureService::new(fake, Duration::from_millis(20));

    let captured = service.capture_selected_text().unwrap();

    assert_eq!(captured.text, "selected text");
}
