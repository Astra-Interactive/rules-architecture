pub(crate) struct PredictionInput {
    pub(crate) symbol: String,
    pub(crate) window: u16,
    pub(crate) features: Vec<f64>,
}
