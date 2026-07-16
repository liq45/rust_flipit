#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
#![windows_subsystem = "windows"]

pub mod resources;
pub mod ini_file;
pub mod display_type;
pub mod location;
pub mod settings;
pub mod timezone_data;
pub mod gdi_helpers;
pub mod screensaver_window;
pub mod settings_window;

use std::env;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Foundation::*;
use windows::core::HSTRING;
use std::result::Result::Ok;

unsafe fn run() {
    // Initialize GDI+ - required before any GDI+ drawing
    let _gdi_token = gdi_helpers::init_gdiplus();

    let args: Vec<String> = env::args().collect();
    let screen_devices = get_screen_info();
    let mut settings = settings::FlipItSettings::load(&screen_devices);

    if args.len() > 1 {
        let fa = &args[1];
        let fl = fa.to_lowercase();
        let (cmd, param) = if fl.len() > 2 && fa.as_bytes().get(2) == Some(&b':') {
            // /c:NNNNNN or /p:NNNNNN format
            let cp = fa.find(':').unwrap_or(2);
            (fl[..cp].to_string(), Some(fl[cp+1..].to_string()))
        } else if fl.len() > 2 && fa.as_bytes().get(1) == Some(&b':') {
            // /:NNNN or /:something format (unlikely but handle)
            (fl[..1].to_string(), Some(fl[2..].to_string()))
        } else { (fl.clone(), args.get(2).cloned()) };
        match cmd.as_str() {
            "/c" => {
                // /c or /c:1234567 - settings dialog (param may have parent hwnd, ignore it)
                settings_window::run_settings(&mut settings);
            }
            "/p" => {
                if let Some(hs) = param {
                    if let Ok(hw) = hs.parse::<isize>() {
                        screensaver_window::ScreensaverWindow::run_preview(&settings, hw);
                    } else {
                        MessageBoxW(None,
                            &HSTRING::from("Sorry, but the expected window handle was not provided."),
                            &HSTRING::from("ScreenSaver"), MB_OK | MB_ICONEXCLAMATION);
                    }
                }
            }
            "/s" => screensaver_window::ScreensaverWindow::run_fullscreen(&settings),
            _ => {
                MessageBoxW(None,
                    &HSTRING::from(&format!("Sorry, but the command line argument \"{}\" is not valid.", fa)),
                    &HSTRING::from("FlipIt"), MB_OK | MB_ICONEXCLAMATION);
            }
        }
    } else { settings_window::run_settings(&mut settings); }

    // Cleanup GDI+ (token dropped = cleanup called)
    if let Some(token) = _gdi_token {
        gdi_helpers::cleanup_gdiplus(token);
    }
}

fn get_screen_info() -> Vec<(i32, String, i32, i32)> {
    unsafe {
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let mut data: Vec<(i32,i32,i32,i32)> = Vec::new();
        let dp = &mut data as *mut _ as isize;
        let _ = EnumDisplayMonitors(None, None, Some(monitor_enum_proc), LPARAM(dp));
        if data.is_empty() { vec![(1, "DISPLAY1".to_string(), sw, sh)] }
        else { data.into_iter().enumerate().map(|(i,(l,t,r,b))| ((i+1)as i32, format!("DISPLAY{}", i+1), r-l, b-t)).collect() }
    }
}

unsafe extern "system" fn monitor_enum_proc(
    _hm: HMONITOR, _hdc: HDC, rect: *mut RECT, data: LPARAM,
) -> windows::core::BOOL {
    if !rect.is_null() {
        let r = &*rect;
        let vec = &mut *(data.0 as *mut Vec<(i32,i32,i32,i32)>);
        vec.push((r.left, r.top, r.right, r.bottom));
    }
    windows::core::BOOL(1)
}

fn main() { unsafe { run(); } }
