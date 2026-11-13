use super::theme::{DARK_ACTIVE, colors, is_dark};
use crate::config::ThemeMode;
use std::sync::{
    OnceLock,
    atomic::{AtomicU8, Ordering},
};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateSolidBrush, FillRect, HBRUSH, HDC, RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE,
    RedrawWindow,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumThreadWindows, GetClientRect, WM_ERASEBKGND, WM_SETTINGCHANGE,
};
use windows::core::{BOOL, PCWSTR, w};

static MODE: AtomicU8 = AtomicU8::new(0);

fn system_is_dark() -> bool {
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
    let mut value = 1u32;
    let mut size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    status.is_ok() && value == 0
}

pub fn set_mode(mode: ThemeMode) {
    MODE.store(mode.index() as u8, Ordering::Relaxed);
    let dark = match mode {
        ThemeMode::System => system_is_dark(),
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
    };
    if DARK_ACTIVE.swap(dark, Ordering::Relaxed) != dark {
        unsafe {
            let _ = EnumThreadWindows(
                windows::Win32::System::Threading::GetCurrentThreadId(),
                Some(refresh_window),
                LPARAM(0),
            );
        }
    }
}

// Six process-lifetime brushes: one per theme and background state. Switching
// themes never invalidates a brush still in use by a native control.
pub fn background_brush(readonly: bool, disabled: bool) -> HBRUSH {
    static BRUSHES: [OnceLock<isize>; 6] = [const { OnceLock::new() }; 6];
    let state = if disabled {
        2
    } else if readonly {
        1
    } else {
        0
    };
    let index = usize::from(is_dark()) * 3 + state;
    let c = colors();
    let color = match state {
        2 => c.disabled_surface,
        1 => c.surface_subtle,
        _ => c.surface,
    };
    HBRUSH(
        *BRUSHES[index].get_or_init(|| unsafe { CreateSolidBrush(color.colorref()).0 as isize })
            as _,
    )
}

pub fn surface_brush() -> HBRUSH {
    background_brush(false, false)
}

unsafe extern "system" fn refresh_child(hwnd: HWND, _: LPARAM) -> BOOL {
    unsafe {
        let _ = windows::Win32::UI::Controls::SetWindowTheme(
            hwnd,
            if is_dark() {
                w!("DarkMode_Explorer")
            } else {
                w!("Explorer")
            },
            PCWSTR::null(),
        );
    }
    BOOL(1)
}

unsafe extern "system" fn refresh_window(hwnd: HWND, _: LPARAM) -> BOOL {
    initialize_window(hwnd);
    BOOL(1)
}

pub fn initialize_window(hwnd: HWND) {
    use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
    let dark = BOOL::from(is_dark());
    unsafe {
        // Older Windows versions may not support dark non-client areas.
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const BOOL).cast(),
            size_of::<BOOL>() as u32,
        );
        let _ = EnumChildWindows(Some(hwnd), Some(refresh_child), LPARAM(0));
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN,
        );
    }
}

pub fn control_background(wparam: WPARAM) -> LRESULT {
    unsafe {
        let hdc = HDC(wparam.0 as _);
        let _ = windows::Win32::Graphics::Gdi::SetTextColor(hdc, colors().text.colorref());
        let _ = windows::Win32::Graphics::Gdi::SetBkColor(hdc, colors().surface.colorref());
    }
    LRESULT(surface_brush().0 as isize)
}

pub fn handle_message(hwnd: HWND, msg: u32, wparam: WPARAM) -> Option<LRESULT> {
    if msg == windows::Win32::UI::WindowsAndMessaging::WM_CTLCOLORLISTBOX {
        return Some(control_background(wparam));
    }
    if msg == WM_SETTINGCHANGE {
        set_mode(ThemeMode::from_index(MODE.load(Ordering::Relaxed) as usize));
    }
    if msg == WM_ERASEBKGND {
        let mut rect = Default::default();
        unsafe {
            let _ = GetClientRect(hwnd, &mut rect);
            FillRect(HDC(wparam.0 as _), &rect, surface_brush());
        }
        return Some(LRESULT(1));
    }
    None
}
