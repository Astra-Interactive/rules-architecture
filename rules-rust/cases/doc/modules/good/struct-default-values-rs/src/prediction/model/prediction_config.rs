// a config read from a file; the defaults keep old files readable
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub(crate) struct PredictionConfig {
    pub(crate) symbol: String,
    pub(crate) window: u16,
    pub(crate) features: Vec<f64>,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            window: 100,
            features: Vec::new(),
        }
    }
}
