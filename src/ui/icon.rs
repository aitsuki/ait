use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    HICON, IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTCOLOR, LR_SHARED, LoadIconW, LoadImageW,
};
use windows::core::PCWSTR;

const APP_ICON_RESOURCE_ID: u16 = 1;

pub(crate) unsafe fn load_app_icon(width: i32, height: i32) -> HICON {
    unsafe {
        GetModuleHandleW(None)
            .and_then(|module| {
                LoadImageW(
                    Some(windows::Win32::Foundation::HINSTANCE(module.0)),
                    PCWSTR(APP_ICON_RESOURCE_ID as usize as *const u16),
                    IMAGE_ICON,
                    width,
                    height,
                    LR_DEFAULTCOLOR | LR_SHARED,
                )
            })
            .map(|handle| HICON(handle.0))
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))
            .unwrap_or_default()
    }
}
