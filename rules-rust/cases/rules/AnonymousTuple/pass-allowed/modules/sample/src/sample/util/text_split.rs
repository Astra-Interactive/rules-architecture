pub(crate) fn split(text: &str, index: usize) -> usize {
    let (head, tail) = text.split_at(index);
    let unit: () = ();
    let single: (u8,) = (1,);
    head.len() + tail.len() + usize::from(single.0) + std::mem::size_of_val(&unit)
}
