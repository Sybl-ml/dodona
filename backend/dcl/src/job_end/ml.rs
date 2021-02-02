//! Handles Machine Learning and Distributed Consensus for the DCL
use crate::job_end::{ClusterInfo, ModelID};

use std::collections::{HashMap, HashSet};

/// Weights the predictions made by models in `model_predictions`
/// based on their errors in validation examples `model_errors`
pub fn weight_predictions(
    model_predictions: HashMap<(ModelID, usize), String>,
    model_errors: HashMap<ModelID, f64>,
) -> (HashMap<ModelID, f64>, Vec<String>) {
    let models: HashSet<ModelID> = model_predictions.keys().map(|(m, _)| m.clone()).collect();

    // Find the inverse of the square error of each model
    let mut weights: HashMap<ModelID, f64> = model_errors
        .iter()
        .map(|(k, v)| (k.to_owned(), 1.0 / (v.powf(2.0))))
        .collect();
    // Normalise weights to sum to 1
    let total: f64 = weights.values().sum();
    weights.values_mut().for_each(|v| *v = *v / total);

    let test_examples: HashSet<&usize> = model_predictions.keys().map(|(_, i)| i).collect();
    let mut indexes: Vec<&usize> = test_examples.into_iter().collect();
    indexes.sort();

    // TODO: implement job type recognition through job config struct
    let job_type = "classification";

    let mut predictions: Vec<String> = vec![];

    match job_type {
        "classification" => {
            for i in indexes.iter() {
                // Add the weight of each model to each possible prediction
                let mut possible: HashMap<&str, f64> = HashMap::new();
                for model in models.iter() {
                    if let Some(prediction) = model_predictions.get(&(model.to_string(), **i)) {
                        let weighting = possible.entry(prediction).or_insert(0.0);
                        *weighting += weights.get(model).unwrap();
                    }
                }
                // Select the prediction with the most weighted votes
                predictions.push(
                    possible
                        .iter()
                        .max_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap())
                        .and_then(|(k, _)| Some(k.to_string()))
                        .unwrap_or("No predictions made".to_owned()),
                );
            }
        }
        _ => {
            for i in indexes.iter() {
                // Create a weighted average taken from all model predictions
                let mut weighted_average: f64 = 0.0;
                for model in models.iter() {
                    if let Some(prediction) = model_predictions.get(&(model.to_string(), **i)) {
                        let value: f64 = prediction.parse().unwrap();
                        weighted_average += value * weights.get(model).unwrap();
                    }
                }
                // The weighted average does not need to be normalised as the weights sum to 1
                predictions.push(weighted_average.to_string());
            }
        }
    }

    (weights, predictions)
}

/// Evaluates the performance of a model based on its test `predictions`,
/// utilising validation answers stored in `info`.
/// Returns a tuple of predictions on test examples and the tuple's validation error
pub fn evaluate_model(
    id: &ModelID,
    predictions: &String,
    info: &ClusterInfo,
) -> (HashMap<usize, String>, f64) {
    // stores the total error penalty for each model
    let mut model_error: f64 = 1.0;
    let mut model_predictions: HashMap<usize, String> = HashMap::new();

    // TODO: implement job type recognition through job config struct
    let job_type = "classification";

    for values in predictions
        .split('\n')
        .map(|s| s.split(',').collect::<Vec<_>>())
    {
        let (record_id, prediction) = (values[0].to_owned(), values[1].to_owned());
        let example = (id.to_owned(), record_id.clone());
        match (info.validation_ans.get(&example), job_type) {
            (Some(answer), "classification") => {
                // if this is a validation response and the job is a classification problem,
                // record an error if the predictions do not match
                if prediction != *answer {
                    model_error += 1.0;
                }
            }
            (Some(answer), _) => {
                // if this is a validation response and the job is a classification problem,
                // record the L2 error of the prediction
                if let (Ok(p), Ok(a)) = (prediction.parse::<f64>(), answer.parse::<f64>()) {
                    model_error += (p - a).powf(2.0);
                }
            }
            (None, _) => {
                // otherwise, record the prediction based on its index in the original dataset
                if let Some(i) = info.prediction_rids.get(&example) {
                    model_predictions.insert(*i, prediction);
                }
            }
        }
    }

    (model_predictions, model_error)
}