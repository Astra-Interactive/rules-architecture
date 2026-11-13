pub(crate) mod clipboard {
    pub(crate) mod api {
        #[cfg(target_os = "linux")]
        pub(crate) use clipboard_linux::Clipboard;
        #[cfg(target_os = "macos")]
        pub(crate) use clipboard_macos::Clipboard;
        #[cfg(target_os = "linux")]
        mod clipboard_linux;
        #[cfg(target_os = "macos")]
        mod clipboard_macos;
    }
}
pub(crate) mod paste {
    pub(crate) mod command {
        pub(crate) use paste_command::run_paste;
        mod paste_command;
    }
}
