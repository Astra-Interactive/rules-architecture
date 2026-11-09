/// Builds the total.
///
/// ---
///
/// ~~~
/// let total = 1;
/// ~~~
// | column | value |
#[doc(hidden)]
#[doc = concat!("built")]
pub(crate) fn build_total() -> u32 {
    let text = "// ==========";
    u32::try_from(text.len()).unwrap_or(0)
}
