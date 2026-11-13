pub(crate) struct ProductId(pub(crate) u64);

pub(crate) trait Shape {
    fn position(&self) -> (i32, i32);
    fn line_items(&self) -> Vec<(ProductId, u32)>;
}

pub(crate) fn measure_label(text: &str, index: usize) -> usize {
    let parts = text.split_at(index);
    parts.0.len() + parts.1.len()
}
