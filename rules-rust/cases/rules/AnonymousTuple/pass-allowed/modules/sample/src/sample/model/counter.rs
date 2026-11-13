pub(crate) struct Counter { pub(crate) count: usize }
impl Iterator for Counter {
    type Item = usize;
    fn next(&mut self) -> Option<usize> { None }
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
    iterator_items!();
}
