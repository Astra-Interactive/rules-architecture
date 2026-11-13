use std::ops::Mul;

pub(crate) fn compute_line_height(font_size: f64, line_height_scale: f64, minimum: f64) -> f64 {
    let line_height = font_size.mul(line_height_scale).round();
    line_height.max(minimum)
}
