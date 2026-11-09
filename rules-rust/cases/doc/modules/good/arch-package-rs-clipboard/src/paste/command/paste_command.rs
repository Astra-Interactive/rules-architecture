use crate::clipboard::api::Clipboard;

pub(crate) fn run_paste(clipboard: &Clipboard) -> usize {
    clipboard.read_text().len()
}
