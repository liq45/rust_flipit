#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::HSTRING;
use windows::core::PCWSTR;
use std::ptr;
use std::result::Result::Ok;

use crate::settings::FlipItSettings;
use crate::settings::ScreenSetting;
use crate::display_type::DisplayType;

static mut DIALOG_SETTINGS: Option<*mut FlipItSettings> = None;
static mut DIALOG_HWND: HWND = HWND(ptr::null_mut());

const IDC_OK: i32 = 1013;
const IDC_CANCEL: i32 = 2;
const IDC_12HR: i32 = 1001;
const IDC_24HR: i32 = 1002;
const IDC_SCALE: i32 = 1003;
const IDC_DST: i32 = 1004;
const IDC_SCREEN_LIST: i32 = 1005;
const IDC_NOTHING: i32 = 1006;
const IDC_CURRENT_TIME: i32 = 1007;
const IDC_WORLD_TIME: i32 = 1008;
const IDC_LOCATIONS: i32 = 1009;
const IDC_CITY: i32 = 1010;
const IDC_ADD: i32 = 1011;
const IDC_REMOVE: i32 = 1012;
const IDC_FONT_COLOR: i32 = 1020;

pub fn run_settings(settings: &mut FlipItSettings) {
    unsafe {
        DIALOG_SETTINGS = Some(settings as *mut FlipItSettings);
        let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let hinstance: HINSTANCE = hinst.into();
        let cls = HSTRING::from("FlipItSettingsDlg");
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.lpfnWndProc = Some(settings_wndproc);
        wc.hInstance = hinstance;
        wc.hCursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        wc.hbrBackground = HBRUSH(1 as _);
        wc.lpszClassName = PCWSTR(cls.as_ptr());
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME, &cls, &HSTRING::from("FlipIt Settings"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT, CW_USEDEFAULT, 590, 570,
            None, None, Some(hinstance), None,
        );
        if let Ok(h) = hwnd {
            DIALOG_HWND = h;
            create_controls(h, settings);
            ShowWindow(h, SW_SHOW);
            UpdateWindow(h);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                if !IsDialogMessageW(h, &msg).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        DIALOG_SETTINGS = None;
        DIALOG_HWND = HWND(ptr::null_mut());
    }
}

unsafe fn cw(ids: i32) -> HWND {
    GetDlgItem(Some(DIALOG_HWND), ids).unwrap_or_default()
}

unsafe fn mk_btn(hwnd: HWND, hi: HMODULE, x: i32, y: i32, w: i32, h: i32, t: &str, ex: u32, id: i32) {
    CreateWindowExW(
        WINDOW_EX_STYLE(0), &HSTRING::from("BUTTON"), &HSTRING::from(t),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | ex),
        x, y, w, h, Some(hwnd), Some(HMENU(id as _)), Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();
}

unsafe fn mk_static(hwnd: HWND, hi: HMODULE, x: i32, y: i32, w: i32, h: i32, t: &str) {
    CreateWindowExW(
        WINDOW_EX_STYLE(0), &HSTRING::from("STATIC"), &HSTRING::from(t),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0),
        x, y, w, h, Some(hwnd), None, Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();
}

unsafe fn create_controls(hwnd: HWND, settings: &FlipItSettings) {
    let hi = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();

    mk_static(hwnd, hi, 16, 12, 100, 20, "General");
    mk_static(hwnd, hi, 43, 42, 100, 17, "Time Display:");
    mk_btn(hwnd, hi, 190, 38, 70, 24, "12 hr", 0x20000, IDC_12HR);
    mk_btn(hwnd, hi, 258, 38, 70, 24, "24 hr", 0, IDC_24HR);
    if settings.display_24h { SendMessageW(cw(IDC_24HR), 0xF1, Some(WPARAM(1)), None); }
    else { SendMessageW(cw(IDC_12HR), 0xF1, Some(WPARAM(1)), None); }

    mk_static(hwnd, hi, 43, 74, 140, 17, "Current Time screens:");
    mk_static(hwnd, hi, 193, 74, 200, 17, "Smaller                      Larger");
    let hs = CreateWindowExW(
        WINDOW_EX_STYLE(0), &HSTRING::from("msctls_trackbar32"), &HSTRING::new(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0),
        196, 92, 170, 30, Some(hwnd), Some(HMENU(IDC_SCALE as _)), Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();
    SendMessageW(hs, 0x416, Some(WPARAM(0)), None); // TBM_SETRANGEMIN
    SendMessageW(hs, 0x417, None, Some(LPARAM(10))); // TBM_SETRANGEMAX
    SendMessageW(hs, 0x415, Some(WPARAM(1)), Some(LPARAM((settings.scale / 10) as isize)));

    mk_static(hwnd, hi, 43, 136, 130, 17, "World Time screens:");
    mk_btn(hwnd, hi, 196, 132, 290, 24, "Show asterisk if city is on daylight time", 0, IDC_DST);
    if settings.show_dst { SendMessageW(cw(IDC_DST), 0xF1, Some(WPARAM(1)), None); }

    mk_static(hwnd, hi, 16, 175, 100, 20, "Screens");

    let hsl = CreateWindowExW(
        WINDOW_EX_STYLE(0), &HSTRING::from("LISTBOX"), &HSTRING::new(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | 0x800000 | 0x200000 | 0x1 | 0x40 | 0x10),
        16, 200, 90, 240, Some(hwnd), Some(HMENU(IDC_SCREEN_LIST as _)), Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();
    for (i, s) in settings.screen_settings.iter().enumerate() {
        let w = wide(&format!("Screen {}", s.screen_number));
        let idx = SendMessageW(hsl, 0x180, Some(WPARAM(0)), Some(LPARAM(w.as_ptr() as isize))); // LB_ADDSTRING
        SendMessageW(hsl, 0x19A, Some(WPARAM(idx.0 as usize)), Some(LPARAM(i as isize))); // LB_SETITEMDATA
    }
    SendMessageW(hsl, 0x18C, Some(WPARAM(0)), None); // LB_SETCURSEL

    let dn = settings.screen_settings.first().map(|s| s.description()).unwrap_or_default();
    let hn = CreateWindowExW(
        WINDOW_EX_STYLE(0), &HSTRING::from("STATIC"), &HSTRING::from(&dn),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0), 117, 200, 200, 17,
        Some(hwnd), None, Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();
    SetWindowTextW(hn, &HSTRING::from(&dn));

    mk_btn(hwnd, hi, 120, 232, 100, 24, "Nothing", 0x20000, IDC_NOTHING);
    mk_btn(hwnd, hi, 120, 262, 100, 24, "Current time", 0, IDC_CURRENT_TIME);
    mk_btn(hwnd, hi, 120, 292, 100, 24, "World Times", 0, IDC_WORLD_TIME);
    if let Some(s) = settings.screen_settings.first() {
        let cid = match s.display_type { DisplayType::None => IDC_NOTHING, DisplayType::CurrentTime => IDC_CURRENT_TIME, DisplayType::WorldTime => IDC_WORLD_TIME };
        SendMessageW(cw(cid), 0xF1, Some(WPARAM(1)), None);
    }

    CreateWindowExW(
        WINDOW_EX_STYLE(0), &HSTRING::from("LISTBOX"), &HSTRING::new(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | 0x800000 | 0x200000),
        225, 232, 230, 120, Some(hwnd), Some(HMENU(IDC_LOCATIONS as _)), Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();

    CreateWindowExW(
        WINDOW_EX_STYLE(0), &HSTRING::from("EDIT"), &HSTRING::new(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | 0x800000 | 0x10000),
        225, 358, 230, 24, Some(hwnd), Some(HMENU(IDC_CITY as _)), Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();

    mk_btn(hwnd, hi, 465, 12, 90, 28, "OK", 0x10000, IDC_OK);
    mk_btn(hwnd, hi, 465, 48, 90, 28, "Cancel", 0x10000, IDC_CANCEL);
    mk_btn(hwnd, hi, 465, 260, 90, 28, "Edit", 0x10000, IDC_REMOVE);
    mk_btn(hwnd, hi, 465, 296, 90, 28, "Remove", 0x10000, IDC_REMOVE);
    mk_btn(hwnd, hi, 465, 390, 90, 28, "Add", 0x10000, IDC_ADD);

    // Font color – hex input box
    mk_static(hwnd, hi, 43, 420, 48, 17, "Color:");
    CreateWindowExW(
        WS_EX_CLIENTEDGE, &HSTRING::from("EDIT"), &HSTRING::new(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | 0x800000 | 0x10000),
        92, 417, 100, 22, Some(hwnd), Some(HMENU(IDC_FONT_COLOR as _)), Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();
    // Set initial text to current color in hex format (RGB display)
    let bgr = settings.font_color & 0xffffff;
    let hex = format!("#{:06X}", rgb_to_bgr(bgr));
    SetWindowTextW(cw(IDC_FONT_COLOR), &HSTRING::from(&hex));
    mk_static(hwnd, hi, 198, 420, 300, 17,
        "Default value: #B7B7B7. Enter any hex color value (e.g. #FF0000).");

    mk_static(hwnd, hi, 160, 450, 290, 20, "https://github.com/liq45/rust_flipit");
}

unsafe fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

/// Parse a hex color string like "#FF0000", "#ff0000", "FF0000", or "ff0000".
/// Returns the color in RGB format (0x00RRGGBB). Returns None on invalid input.
fn parse_hex_color(s: &str) -> Option<u32> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 { return None; }
    u32::from_str_radix(s, 16).ok().map(|v| v & 0xffffff)
}

/// Convert RGB (0x00RRGGBB) to GDI BGR (0x00BBGGRR) — or vice versa (it's symmetric).
fn rgb_to_bgr(c: u32) -> u32 {
    let r = c & 0xff;
    let g = (c >> 8) & 0xff;
    let b = (c >> 16) & 0xff;
    (r << 16) | (g << 8) | b
}

unsafe extern "system" fn settings_wndproc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as i32;
            match id {
                IDC_OK => {
                    let pos = SendMessageW(cw(IDC_SCALE), 0x400, Some(WPARAM(0)), None).0 as i32;
                    // Read hex color from edit box
                    let mut buf = [0u16; 128];
                    let len = GetWindowTextW(cw(IDC_FONT_COLOR), &mut buf);
                    let text = String::from_utf16_lossy(&buf[..len as usize]);
                    let color = parse_hex_color(text.trim()).unwrap_or(0x00b7b7b7);
                    if let Some(s) = DIALOG_SETTINGS.as_mut() {
                        (**s).scale = pos * 10;
                        (**s).font_color = rgb_to_bgr(color);
                        (**s).save();
                    }
                    PostQuitMessage(0); LRESULT(0)
                }
                IDC_CANCEL => { PostQuitMessage(0); LRESULT(0) }
                IDC_12HR | IDC_24HR => {
                    if let Some(s) = DIALOG_SETTINGS.as_mut() { (**s).display_24h = id == IDC_24HR; }
                    LRESULT(0)
                }
                IDC_DST => {
                    let c = SendMessageW(cw(IDC_DST), 0xF0, Some(WPARAM(0)), None).0 != 0;
                    if let Some(s) = DIALOG_SETTINGS.as_mut() { (**s).show_dst = c; }
                    LRESULT(0)
                }
                IDC_NOTHING | IDC_CURRENT_TIME | IDC_WORLD_TIME => {
                    let sel = SendMessageW(cw(IDC_SCREEN_LIST), 0x188, Some(WPARAM(0)), None).0 as usize;
                    if let Some(s) = DIALOG_SETTINGS.as_mut() {
                        if sel < (**s).screen_settings.len() {
                            let screen_ptr = &mut (**s).screen_settings as *mut Vec<ScreenSetting>;
                            (&mut (*screen_ptr))[sel].display_type = match id {
                                IDC_NOTHING => DisplayType::None,
                                IDC_CURRENT_TIME => DisplayType::CurrentTime,
                                _ => DisplayType::WorldTime,
                            };
                        }
                    }
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
        WM_DESTROY | WM_CLOSE => { PostQuitMessage(0); LRESULT(0) }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
