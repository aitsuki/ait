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
