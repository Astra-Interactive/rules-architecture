use crate::prediction::model::PredictionInput;

pub(crate) fn build_prediction_inputs(symbols: &[&str]) -> Vec<PredictionInput> {
    let mut inputs = Vec::new();
    for &symbol in symbols {
        let input = PredictionInput {
            symbol: symbol.to_owned(),
            window: 100,
            features: Vec::new(),
        };
        inputs.push(input);
    }
    inputs
}
