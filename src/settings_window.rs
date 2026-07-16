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
const IDC_FONT_COLOR0: i32 = 1020;
const IDC_FONT_COLOR1: i32 = 1021;
const IDC_FONT_COLOR2: i32 = 1022;
const IDC_FONT_COLOR3: i32 = 1023;
const IDC_FONT_COLOR4: i32 = 1024;
const IDC_FONT_COLOR5: i32 = 1025;
const IDC_FONT_COLOR6: i32 = 1026;
const IDC_FONT_ALPHA: i32 = 1027;

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

    // Font color & transparency – placed above GitHub link
    mk_static(hwnd, hi, 43, 420, 130, 17, "Font Color:");
    let color_btns: [(u32, &str, i32); 7] = [
        (0x00b7b7b7, "Def", IDC_FONT_COLOR0), (0x00ffffff, "Wht", IDC_FONT_COLOR1),
        (0x0000ff00, "Grn", IDC_FONT_COLOR2), (0x000000ff, "Red", IDC_FONT_COLOR3),
        (0x00ff0000, "Blu", IDC_FONT_COLOR4), (0x00ffff00, "Cyn", IDC_FONT_COLOR5),
        (0x00808080, "Gry", IDC_FONT_COLOR6),
    ];
    for (i, &(_bgr, label, btn_id)) in color_btns.iter().enumerate() {
        let (row, col) = (i / 4, i % 4);
        let bx = 43 + (col as i32) * 48;
        let by = 440 + (row as i32) * 26;
        let hb = CreateWindowExW(WINDOW_EX_STYLE(0), &HSTRING::from("BUTTON"), &HSTRING::from(label),
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0), bx, by, 44, 22,
            Some(hwnd), Some(HMENU(btn_id as _)), Some(HINSTANCE(hi.0)), None,
        ).unwrap_or_default();
        let bf = CreateFontW(11, 0, 0, 0, 0, 0, 0, 0, DEFAULT_CHARSET,
            FONT_OUTPUT_PRECISION(OUT_TT_ONLY_PRECIS.0), FONT_CLIP_PRECISION(CLIP_DEFAULT_PRECIS.0),
            FONT_QUALITY(CLEARTYPE_QUALITY.0), 0, &HSTRING::from("Segoe UI"));
        if !bf.0.is_null() { SendMessageW(hb, 0x30, Some(WPARAM(bf.0 as usize)), None); }
    }
    mk_static(hwnd, hi, 43, 498, 130, 17, "Transparency:");
    let ha = CreateWindowExW(WINDOW_EX_STYLE(0), &HSTRING::from("msctls_trackbar32"), &HSTRING::new(),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0), 150, 496, 200, 30,
        Some(hwnd), Some(HMENU(IDC_FONT_ALPHA as _)), Some(HINSTANCE(hi.0)), None,
    ).unwrap_or_default();
    SendMessageW(ha, 0x416, Some(WPARAM(0)), None);
    SendMessageW(ha, 0x417, None, Some(LPARAM(10)));
    SendMessageW(ha, 0x415, Some(WPARAM(1)), Some(LPARAM((settings.font_alpha / 25) as isize)));

    mk_static(hwnd, hi, 160, 530, 290, 20, "https://github.com/liq45/rust_flipit");
}

unsafe fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

unsafe fn set_font_color(idx: usize) {
    let colors: [u32; 7] = [0x00b7b7b7, 0x00ffffff, 0x0000ff00, 0x000000ff, 0x00ff0000, 0x00ffff00, 0x00cccccc];
    if idx < 7 {
        if let Some(s) = DIALOG_SETTINGS.as_mut() { (**s).font_color = colors[idx]; }
    }
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
                    let alpha_pos = SendMessageW(cw(IDC_FONT_ALPHA), 0x400, Some(WPARAM(0)), None).0 as i32;
                    if let Some(s) = DIALOG_SETTINGS.as_mut() {
                        (**s).scale = pos * 10;
                        (**s).font_alpha = (alpha_pos * 25).min(255) as u32;
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
                IDC_FONT_COLOR0 => { set_font_color(0); LRESULT(0) }
                IDC_FONT_COLOR1 => { set_font_color(1); LRESULT(0) }
                IDC_FONT_COLOR2 => { set_font_color(2); LRESULT(0) }
                IDC_FONT_COLOR3 => { set_font_color(3); LRESULT(0) }
                IDC_FONT_COLOR4 => { set_font_color(4); LRESULT(0) }
                IDC_FONT_COLOR5 => { set_font_color(5); LRESULT(0) }
                IDC_FONT_COLOR6 => { set_font_color(6); LRESULT(0) }
                IDC_FONT_ALPHA => { LRESULT(0) }
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
