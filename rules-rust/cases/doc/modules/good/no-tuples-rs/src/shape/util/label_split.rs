pub(crate) fn rotate_label(text: &str, index: usize) -> String {
    let (head, tail) = text.split_at(index);
    format!("{tail}{head}")
}
