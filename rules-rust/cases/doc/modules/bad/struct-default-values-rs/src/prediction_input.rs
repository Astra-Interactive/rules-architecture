#[derive(Default)]
pub(crate) struct PredictionInput {
    pub(crate) symbol: String,
    pub(crate) window: u16,
    pub(crate) features: Vec<f64>,
}

pub(crate) fn build_window() -> u16 {
    let input = PredictionInput {
        window: 100,
        ..Default::default()
    };
    input.window
}
