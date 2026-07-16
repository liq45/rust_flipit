pub fn init_gdiplus() -> Option<usize> {
    unsafe {
        let mut token: usize = 0;
        let input = windows::Win32::Graphics::GdiPlus::GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: 0,
            SuppressBackgroundThread: false.into(),
            SuppressExternalCodecs: false.into(),
        };
        let mut output = windows::Win32::Graphics::GdiPlus::GdiplusStartupOutput::default();
        let status = windows::Win32::Graphics::GdiPlus::GdiplusStartup(&mut token, &input, &mut output);
        if status.0 == 0 { Some(token) } else { None }
    }
}

pub fn cleanup_gdiplus(token: usize) {
    unsafe { windows::Win32::Graphics::GdiPlus::GdiplusShutdown(token); }
}
