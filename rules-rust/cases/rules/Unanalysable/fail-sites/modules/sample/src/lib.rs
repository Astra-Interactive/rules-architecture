pub(crate) mod sample {
    mod broken_file;
    mod missing_file;
    #[cfg_attr(unix, path = "unix.rs")]
    mod platform;
    include!("items.rs");
}
generate_items!();
