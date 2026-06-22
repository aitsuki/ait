#[test]
fn iss_script_requests_running_ait_exit_before_installing() {
    let script = std::fs::read_to_string("installer/ait.iss").expect("read installer script");

    assert!(script.contains("CloseApplications=force"));
    assert!(script.contains("function PrepareToInstall(var NeedsRestart: Boolean): String"));
    assert!(script.contains("AitTrayWindowClassName = 'ait_tray_window'"));
    assert!(script.contains("WM_TRAY_COMMAND = $8014"));
    assert!(script.contains("MENU_EXIT = 1004"));
    assert!(script.contains("PostMessage(Window, WM_TRAY_COMMAND, MENU_EXIT, 0)"));
    assert!(script.contains("if Window = 0 then"));
    assert!(script.contains("Exit;"));
    assert!(!script.contains("function InitializeSetup"));
    assert!(!script.contains("PostMessage(Window, WM_CLOSE"));
}
