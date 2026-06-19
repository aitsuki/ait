#[test]
fn release_windows_build_uses_gui_subsystem() {
    let main_rs = std::fs::read_to_string("src/main.rs").expect("read src/main.rs");

    assert!(
        main_rs.contains("#![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]"),
        "release builds should use the Windows GUI subsystem so launching ait.exe does not open a console window"
    );
}
