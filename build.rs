fn main() {
    // Embed icon and version info into the PE binary using winresource
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        winresource::WindowsResource::new()
            .set_icon("res/favicon.ico")
            .set("FileDescription", "FlipIt Screensaver")
            .set("ProductName", "rust_flipit")
            .set("LegalCopyright", "CC0 1.0")
            .compile()
            .expect("Failed to embed resources");
    }
}
