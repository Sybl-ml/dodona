//! Handles Machine Learning and Distributed Consensus for the DCL
use crate::job_end::{
    ClusterInfo, ModelErrors, ModelID, ModelPredictions, ModelWeights, Predictions,
};
use models::job_performance::JobPerformance;
use models::jobs::PredictionType;
use mongodb::{
    bson::{document::Document, oid::ObjectId},
    Database,
};

use anyhow::Result;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::node_end::NodePool;

/// Weights the predictions made by models in `model_predictions`
/// based on their errors in validation examples `model_errors`
pub fn weight_predictions(
    model_predictions: &ModelPredictions,
    model_errors: &ModelErrors,
    info: &ClusterInfo,
) -> (ModelWeights, Vec<String>) {
    let models: HashSet<ModelID> = model_predictions.keys().map(|(m, _)| m.clone()).collect();

    // Find the inverse of the square error of each non-penalised model
    let mut weights: HashMap<ModelID, f64> = model_errors
        .iter()
        .filter_map(|(k, v)| {
            v.is_some()
                .then(|| (k.to_owned(), 1.0 / (v.unwrap().powf(2.0))))
        })
        .collect();
    // Normalise weights to sum to 1
    let total: f64 = weights.values().sum();
    weights.values_mut().for_each(|v| *v /= total);

    let test_examples: HashSet<&usize> = model_predictions.keys().map(|(_, i)| i).collect();
    let mut indexes: Vec<&usize> = test_examples.into_iter().collect();
    indexes.sort();

    let job_type = info.job.config.prediction_type;

    let mut predictions: Vec<String> = Vec::new();
    predictions.push(String::from("predicted"));

    match job_type {
        PredictionType::Classification => {
            for i in &indexes {
                // Add the weight of each model to each possible prediction
                let mut possible: HashMap<&str, f64> = HashMap::new();
                for model in &models {
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
                        .map_or_else(
                            || String::from("No predictions made"),
                            |(k, _)| (*k).to_string(),
                        ),
                );
            }
        }
        PredictionType::Regression => {
            for i in &indexes {
                // Create a weighted average taken from all model predictions
                let mut weighted_average: f64 = 0.0;
                for model in &models {
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
    model_id: &str,
    predictions: &str,
    info: &ClusterInfo,
) -> Option<(Predictions, f64)> {
    // stores the total error penalty for each model
    let mut model_error: f64 = 1.0;
    let mut model_predictions: Predictions = HashMap::new();

    let job_type = info.job.config.prediction_type;

    let mut predictions: Vec<_> = predictions.trim().split('\n').collect();

    // if the model returned predictions with a header, ignore them
    if predictions[0].contains("record_id") {
        predictions = predictions[1..].to_vec();
    }

    for values in predictions.iter().map(|s| s.split(',').collect::<Vec<_>>()) {
        if values.len() != 2 {
            return None;
        }
        let (record_id, prediction) = (values[0].to_owned(), values[1].to_owned());
        let example = (model_id.to_owned(), record_id.clone());
        match (info.validation_ans.get(&example), job_type) {
            (Some(answer), PredictionType::Classification) => {
                // if this is a validation response and the job is a classification problem,
                // record an error if the predictions do not match
                if prediction != *answer {
                    model_error += 1.0;
                }
            }
            (Some(answer), PredictionType::Regression) => {
                // if this is a validation response and the job is a classification problem,
                // record the L2 error of the prediction
                if let (Ok(p), Ok(a)) = (prediction.parse::<f64>(), answer.parse::<f64>()) {
                    model_error += (p - a).powf(2.0);
                } else {
                    return None;
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

    let predicted: HashSet<&str> = predictions
        .iter()
        .map(|s| s.split(',').next().unwrap())
        .collect();

    if predicted
        == info
            .validation_ans
            .keys()
            .chain(info.prediction_rids.keys())
            .filter_map(|(m, r)| (m == model_id).then(|| r.as_str()))
            .collect()
    {
        Some((model_predictions, model_error))
    } else {
        None
    }
}

/// Function for calculating model performance
///
/// Will take in a HashMap of model ids and their
/// weight in the ensemble model. It will then
/// calculate their performance on the problem
/// and will upload it to the database.
pub async fn model_performance(
    database: Arc<Database>,
    weights: ModelWeights,
    project_id: &ObjectId,
    nodepool: Option<Arc<NodePool>>,
) -> Result<()> {
    let job_performances = database.collection("job_performances");
    let model_num = weights.len();
    let mut job_perf_vec: Vec<Document> = Vec::new();

    for (model, weight) in &weights {
        let val = (weight * model_num as f64) - 1.0;
        let perf: f64 = 0.5 * ((2.0 * val).tanh()) + 0.5;

        log::info!(
            "Model with id={} has weight={} and performance={}",
            model,
            weight,
            perf
        );

        let job_performance = JobPerformance::new(
            project_id.clone(),
            ObjectId::with_string(&model).unwrap(),
            perf,
        );

        if let Some(np) = &nodepool {
            np.update_node_performance(&model, perf).await;
        }

        job_perf_vec.push(mongodb::bson::ser::to_document(&job_performance).unwrap());
    }

    if job_perf_vec.is_empty() {
        log::warn!(
            "No models returned correct predictions for project_id={}",
            project_id
        );
    } else {
        job_performances.insert_many(job_perf_vec, None).await?;
    }

    Ok(())
}

/// Function for penalising a list of malicious models
///
/// Penalises a list of models with a performance of 0 for a given job
/// based on the detection of malicious behaviour in their predictions
pub async fn penalise(
    database: Arc<Database>,
    models: Vec<ModelID>,
    project_id: &ObjectId,
    nodepool: Option<Arc<NodePool>>,
) -> Result<()> {
    let job_performances = database.collection("job_performances");
    let mut job_perf_vec: Vec<Document> = Vec::new();

    for model in models {
        let perf: f64 = 0.0;

        log::warn!(
            "Model with id={} is being penalised for malicious behaviour, performance={}",
            model,
            perf
        );

        let job_performance = JobPerformance::new(
            project_id.clone(),
            ObjectId::with_string(&model).unwrap(),
            perf,
        );

        if let Some(np) = &nodepool {
            np.update_node_performance(&model, perf).await;
        }

        job_perf_vec.push(mongodb::bson::ser::to_document(&job_performance).unwrap());
    }

    if job_perf_vec.is_empty() {
        log::info!("No models were penalised for project_id={}", project_id);
    } else {
        job_performances.insert_many(job_perf_vec, None).await?;
    }

    Ok(())
}
