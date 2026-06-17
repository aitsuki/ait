#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    TranslateSelection,
    TranslateClipboard,
    OpenSettings,
    RetryTranslation,
    CopyTranslation,
    Exit,
}
