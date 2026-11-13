mod api {
    #[cfg(target_os = "linux")]
    mod clipboard;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    mod clipboard_other;
    #[cfg(unix)]
    fn run() {}
}
