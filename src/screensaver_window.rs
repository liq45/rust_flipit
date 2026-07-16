#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Graphics::GdiPlus::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::HSTRING;
use windows::core::PCWSTR;
use std::ptr;
use std::result::Result::Ok;
use std::mem;
use chrono::Local;

use crate::settings::FlipItSettings;

/// Global settings for rendering, set once on startup.
static mut GFX_SCALE: i32 = 100;
static mut FONT_COLOR_ARGB: u32 = 0xffb7b7b7;

pub struct ScreensaverWindow;

impl ScreensaverWindow {
    pub fn run_fullscreen(settings: &FlipItSettings) {
        unsafe {
            GFX_SCALE = settings.scale;
            FONT_COLOR_ARGB = ((settings.font_alpha as u32) << 24) | (settings.font_color & 0xffffff);
            let sw = GetSystemMetrics(SM_CXSCREEN);
            let sh = GetSystemMetrics(SM_CYSCREEN);
            let monitors = vec![(0, 0, sw, sh)];

            let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
            let hinstance: HINSTANCE = hinst.into();
            let cls = register_screensaver_class(hinstance);

            let mut handles: Vec<HWND> = Vec::new();
            for (i, &(l, t, r, b)) in monitors.iter().enumerate() {
                if i >= settings.screen_settings.len() { continue; }
                if settings.screen_settings[i].display_type as i32 == 0 { continue; }
                let h = CreateWindowExW(
                    WS_EX_TOPMOST, &cls, &HSTRING::new(), WS_POPUP | WS_VISIBLE,
                    l, t, r-l, b-t, None, None, Some(hinstance), None,
                );
                if let Ok(hw) = h {
                    handles.push(hw);
                    ShowWindow(hw, SW_SHOW);
                    UpdateWindow(hw);
                }
            }
            if !handles.is_empty() {
                for &h in &handles { SetTimer(Some(h), 1, 1000, None); }
                let mut msg = MSG::default();
                loop {
                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                        if msg.message == WM_QUIT { return; }
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }

    pub fn run_preview(_settings: &FlipItSettings, parent_hwnd: isize) {
        unsafe {
            GFX_SCALE = _settings.scale;
            FONT_COLOR_ARGB = ((_settings.font_alpha as u32) << 24) | (_settings.font_color & 0xffffff);
            let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
            let hinstance: HINSTANCE = hinst.into();
            let cls = register_preview_class(hinstance);
            let parent = HWND(parent_hwnd as *mut _);
            let mut r = RECT::default();
            GetClientRect(parent, &mut r);
            let w = r.right - r.left; let h = r.bottom - r.top;
            if let Ok(child) = CreateWindowExW(
                WINDOW_EX_STYLE(0), &cls, &HSTRING::new(),
                WS_CHILD | WS_VISIBLE, 0, 0, w, h,
                Some(parent), None, Some(hinstance), None,
            ) {
                SetTimer(Some(child), 1, 1000, None);
                let mut msg = MSG::default();
                loop {
                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                        if msg.message == WM_QUIT { return; }
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    if !IsWindow(Some(parent)).as_bool() { return; }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }
}

unsafe fn register_screensaver_class(hinstance: HINSTANCE) -> HSTRING {
    let name = HSTRING::from("FlipItScreensaver");
    let mut wc: WNDCLASSW = std::mem::zeroed();
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = Some(screensaver_wndproc);
    wc.hInstance = hinstance;
    wc.hCursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
    wc.hbrBackground = HBRUSH(GetStockObject(BLACK_BRUSH).0 as _);
    wc.lpszClassName = PCWSTR(name.as_ptr());
    RegisterClassW(&wc);
    name
}

unsafe fn register_preview_class(hinstance: HINSTANCE) -> HSTRING {
    let name = HSTRING::from("FlipItPreview");
    let mut wc: WNDCLASSW = std::mem::zeroed();
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = Some(preview_wndproc);
    wc.hInstance = hinstance;
    wc.hCursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
    wc.hbrBackground = HBRUSH(GetStockObject(BLACK_BRUSH).0 as _);
    wc.lpszClassName = PCWSTR(name.as_ptr());
    RegisterClassW(&wc);
    name
}

unsafe extern "system" fn screensaver_wndproc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Store initial cursor position packed: hiword=lparam_x, loword=lparam_y
            // Using GWLP_USERDATA to remember initialization state per-window
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            draw_clock_gdi(hwnd, hdc, false);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_TIMER => { let _ = InvalidateRect(Some(hwnd), None, false); LRESULT(0) }
        WM_MOUSEMOVE => {
            // Skip initial spurious WM_MOUSEMOVE events during window creation.
            // GWLP_USERDATA stores a counter: 0 = not started, then 3,2,1 = ignore,
            // then real movement detection.
            let counter = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if counter < 3 {
                // Still in initialization phase - increment and ignore
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, counter + 1);
                LRESULT(0)
            } else if counter == 3 {
                // Transition: store actual cursor position for comparison
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, lparam.0);
                LRESULT(0)
            } else if counter == lparam.0 {
                // Same cursor position - not real movement
                LRESULT(0)
            } else {
                // Real mouse movement detected - exit
                PostQuitMessage(0);
                LRESULT(0)
            }
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => { PostQuitMessage(0); LRESULT(0) }
        WM_KEYDOWN | WM_SYSKEYDOWN => { PostQuitMessage(0); LRESULT(0) }
        WM_DESTROY => { PostQuitMessage(0); LRESULT(0) }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn preview_wndproc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            draw_clock_gdi(hwnd, hdc, true);
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_TIMER => { InvalidateRect(Some(hwnd), None, false); LRESULT(0) }
        WM_DESTROY => { PostQuitMessage(0); LRESULT(0) }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn draw_clock_gdi(hwnd: HWND, hdc: HDC, _is_preview: bool) {
    let mut r = RECT::default();
    let _ = GetClientRect(hwnd, &mut r);
    let cw = r.right - r.left;
    let ch = r.bottom - r.top;
    if cw <= 0 || ch <= 0 { return; }

    // ---- Double buffering ----
    let mem_dc = CreateCompatibleDC(Some(hdc));
    let bmp = CreateCompatibleBitmap(hdc, cw, ch);
    let old_bmp = SelectObject(mem_dc, HGDIOBJ(bmp.0));
    // Black background via FillRect
    let black_brush = HBRUSH(GetStockObject(BLACK_BRUSH).0 as _);
    let mut rr = RECT { left: 0, top: 0, right: cw, bottom: ch };
    FillRect(mem_dc, &mut rr as *mut RECT as *const RECT, black_brush);

    // Layout -- box = 90% screen height, ignore width constraint
    let box_sz = (ch as f64 * 0.90) as i32;

    let gap = (box_sz as f64 * 0.05) as i32;  // BoxSeparationPercent = 0.05
    let tot = box_sz * 2 + gap;
    let bx = (cw - tot) / 2;
    let by = (ch - box_sz) / 2;
    let radius = (box_sz / 20).max(2);

    // ---- GDI+ gradient boxes ----
    let mut gfx: *mut GpGraphics = ptr::null_mut();
    if GdipCreateFromHDC(mem_dc, &mut gfx) == Status(0) && !gfx.is_null() {
        GdipSetSmoothingMode(gfx, SmoothingModeHighQuality);
        let rad = radius as f32;
        let (bxf, byf) = (bx as f32, by as f32);
        let szf = box_sz as f32;

        // Box 1 - hour
        let rf1 = RectF { X: bxf, Y: byf, Width: szf, Height: szf };
        let mut gb1: *mut GpLineGradient = ptr::null_mut();
        if GdipCreateLineBrushFromRect(&rf1, 0xff121212, 0xff0a0a0a, LinearGradientModeVertical, WrapModeTile, &mut gb1) == Status(0) && !gb1.is_null() {
            let mut p1: *mut GpPath = ptr::null_mut();
            if GdipCreatePath(FillModeAlternate, &mut p1) == Status(0) && !p1.is_null() {
                rounded_rect(p1, bxf, byf, szf, szf, rad);
                GdipFillPath(gfx, gb1 as *mut GpBrush, p1);
                GdipDeletePath(p1);
            }
            let mut pn1: *mut GpPen = ptr::null_mut();
            if GdipCreatePen1(0xff000000, 4.0, UnitPixel, &mut pn1) == Status(0) && !pn1.is_null() {
                GdipDrawLine(gfx, pn1, bxf, byf + szf / 2.0, bxf + szf, byf + szf / 2.0);
                GdipDeletePen(pn1);
            }
            GdipDeleteBrush(gb1 as *mut GpBrush);
        }

        // Box 2 - minute
        let sx2 = (bx + box_sz + gap) as f32;
        let rf2 = RectF { X: sx2, Y: byf, Width: szf, Height: szf };
        let mut gb2: *mut GpLineGradient = ptr::null_mut();
        if GdipCreateLineBrushFromRect(&rf2, 0xff121212, 0xff0a0a0a, LinearGradientModeVertical, WrapModeTile, &mut gb2) == Status(0) && !gb2.is_null() {
            let mut p2: *mut GpPath = ptr::null_mut();
            if GdipCreatePath(FillModeAlternate, &mut p2) == Status(0) && !p2.is_null() {
                rounded_rect(p2, sx2, byf, szf, szf, rad);
                GdipFillPath(gfx, gb2 as *mut GpBrush, p2);
                GdipDeletePath(p2);
            }
            let mut pn2: *mut GpPen = ptr::null_mut();
            if GdipCreatePen1(0xff000000, 4.0, UnitPixel, &mut pn2) == Status(0) && !pn2.is_null() {
                GdipDrawLine(gfx, pn2, sx2, byf + szf / 2.0, sx2 + szf, byf + szf / 2.0);
                GdipDeletePen(pn2);
            }
            GdipDeleteBrush(gb2 as *mut GpBrush);
        }
        GdipDeleteGraphics(gfx);
    }

    // ---- GDI text ----
    let now = Local::now();
    let hstr: Vec<u16> = now.format("%H").to_string().encode_utf16().collect();
    let mstr: Vec<u16> = now.format("%M").to_string().encode_utf16().collect();

    let fs = (box_sz as f64 * 0.78) as i32;
    let hfont = CreateFontW(
        -fs, 0, 0, 0, 700, 0, 0, 0,
        DEFAULT_CHARSET,
        FONT_OUTPUT_PRECISION(OUT_TT_ONLY_PRECIS.0),
        FONT_CLIP_PRECISION(CLIP_DEFAULT_PRECIS.0),
        FONT_QUALITY(CLEARTYPE_QUALITY.0),
        0, &HSTRING::from("Segoe UI"),
    );
    if !hfont.0.is_null() {
        let old_f = SelectObject(mem_dc, HGDIOBJ(hfont.0));
        SetBkMode(mem_dc, TRANSPARENT);
        SetTextColor(mem_dc, windows::Win32::Foundation::COLORREF(FONT_COLOR_ARGB & 0xffffff));
        let df = DRAW_TEXT_FORMAT(DT_CENTER.0 | DT_VCENTER.0 | DT_SINGLELINE.0);

        let mut hr1 = RECT { left: bx, top: by, right: bx + box_sz, bottom: by + box_sz };
        let mut hb = hstr.clone();
        DrawTextW(mem_dc, &mut hb, &mut hr1, df);

        let mut hr2 = RECT { left: bx + box_sz + gap, top: by, right: bx + tot, bottom: by + box_sz };
        let mut mb = mstr.clone();
        DrawTextW(mem_dc, &mut mb, &mut hr2, df);

        SelectObject(mem_dc, old_f);
        let _ = DeleteObject(HGDIOBJ(hfont.0));
    }

    // ---- Flip to screen ----
    let _ = BitBlt(hdc, 0, 0, cw, ch, Some(mem_dc), 0, 0, SRCCOPY);

    // ---- Cleanup ----
    SelectObject(mem_dc, old_bmp);
    let _ = DeleteObject(HGDIOBJ(bmp.0));
    let _ = DeleteDC(mem_dc);
}

unsafe fn rounded_rect(path: *mut GpPath, x: f32, y: f32, w: f32, h: f32, r: f32) {
    GdipAddPathArc(path, x, y, r*2.0, r*2.0, 180.0, 90.0);
    GdipAddPathLine(path, x+r, y, x+w-r, y);
    GdipAddPathArc(path, x+w-r*2.0, y, r*2.0, r*2.0, 270.0, 90.0);
    GdipAddPathLine(path, x+w, y+r, x+w, y+h-r);
    GdipAddPathArc(path, x+w-r*2.0, y+h-r*2.0, r*2.0, r*2.0, 0.0, 90.0);
    GdipAddPathLine(path, x+w-r, y+h, x+r, y+h);
    GdipAddPathArc(path, x, y+h-r*2.0, r*2.0, r*2.0, 90.0, 90.0);
    GdipAddPathLine(path, x, y+h-r, x, y+r);
    GdipClosePathFigure(path);
}
