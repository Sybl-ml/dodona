use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::Arc;
use std::time::Duration;

use float_cmp::approx_eq;
use futures::stream::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};

use dcl::job_end::finance::Pricing;
use dcl::job_end::ml::{evaluate_model, model_performance, penalise, weight_predictions};
use dcl::job_end::{ClusterInfo, ModelID, WriteBackMemory};
use messages::ClientMessage;
use models::jobs::PredictionType;
use models::users::User;

mod common;

#[tokio::test]
async fn test_write_back_predictions() {
    let model_id: ModelID = ModelID::from("ModelID1");
    let val1 = String::from("pred1");

    let mut pred_map = HashMap::new();
    pred_map.insert(1, val1.clone());

    let wb: WriteBackMemory = WriteBackMemory::new();

    let wb_clone = wb.clone();
    let pred_map_clone = pred_map.clone();
    let mid_clone = model_id.clone();

    tokio::spawn(async move {
        wb_clone.write_predictions(mid_clone, pred_map_clone);
    });

    tokio::time::sleep(Duration::from_millis(1)).await;

    let predictions = wb.get_predictions();
    let pred_val = predictions.get(&(model_id, 1)).unwrap();
    assert_eq!(&val1, pred_val);
}

#[tokio::test]
async fn test_write_back_errors() {
    let model_id: ModelID = ModelID::from("ModelID1");
    let error: Option<f64> = Some(10.0);

    let wb: WriteBackMemory = WriteBackMemory::new();

    let wb_clone = wb.clone();

    let mid_clone = model_id.clone();

    tokio::spawn(async move {
        wb_clone.write_error(mid_clone, error);
    });

    tokio::time::sleep(Duration::from_millis(1)).await;

    let errors = wb.get_errors();
    let error_val = errors.get(&model_id).unwrap();
    assert_eq!(&error, error_val);
}

#[test]
fn test_evaluate_model() {
    let id = ModelID::from("ModelID1");

    let validation = vec![
        ((id.clone(), "1".to_string()), "1".to_string()),
        ((id.clone(), "2".to_string()), "2".to_string()),
        ((id.clone(), "3".to_string()), "3".to_string()),
        ((id.clone(), "4".to_string()), "4".to_string()),
    ];
    let rids = vec![
        ((id.clone(), "5".to_string()), 1),
        ((id.clone(), "6".to_string()), 2),
        ((id.clone(), "7".to_string()), 3),
        ((id.clone(), "8".to_string()), 4),
    ];
    let test = vec![
        (1, "5".to_string()),
        (2, "6".to_string()),
        (3, "7".to_string()),
        (4, "8".to_string()),
    ];

    let validation_ans: HashMap<(ModelID, String), String> =
        HashMap::from_iter(validation.into_iter());
    let prediction_rids: HashMap<(ModelID, String), usize> = HashMap::from_iter(rids.into_iter());
    let test_predictions: HashMap<usize, String> = HashMap::from_iter(test.into_iter());

    let info = ClusterInfo {
        project_id: ObjectId::with_string(common::USER_ID).unwrap(),
        columns: HashMap::new(),
        config: ClientMessage::Alive { timestamp: 0 },
        validation_ans: validation_ans,
        prediction_rids: prediction_rids,
    };

    let predictions = "2,3\n3,2\n4,1\n5,0\n6,0\n7,0\n8,0".to_owned();
    assert!(evaluate_model(&id, &predictions, &info).is_none());

    let predictions = "1,4\n2,3\n3,2\n4,1\n5,0\n6,0\n7,0".to_owned();
    assert!(evaluate_model(&id, &predictions, &info).is_none());

    let predictions = "1,4\n2,3\n3,2\n4,1\n5,0\n6,0\n7,0\n8,0".to_owned();
    let (_, model_error) = evaluate_model(&id, &predictions, &info).unwrap();
    assert!(approx_eq!(f64, model_error, 5.0, ulps = 2));

    let predictions = "1,1\n2,3\n3,2\n4,4\n5,0\n6,0\n7,0\n8,0".to_owned();
    let (_, model_error) = evaluate_model(&id, &predictions, &info).unwrap();
    assert!(approx_eq!(f64, model_error, 3.0, ulps = 2));

    let predictions = "1,1\n2,2\n3,3\n4,4\n5,5\n6,6\n7,7\n8,8".to_owned();
    let (model_predictions, model_error) = evaluate_model(&id, &predictions, &info).unwrap();
    assert!(approx_eq!(f64, model_error, 1.0, ulps = 2));
    assert_eq!(model_predictions, test_predictions);
}

#[test]
fn test_weight_predictions() {
    let ids = vec![
        ModelID::from("Good"),
        ModelID::from("Bad"),
        ModelID::from("Ugly"),
    ];

    let validation = ids
        .iter()
        .map(|id| {
            vec![
                ((id.clone(), "1"), "1"),
                ((id.clone(), "2"), "2"),
                ((id.clone(), "3"), "3"),
                ((id.clone(), "4"), "4"),
            ]
        })
        .flatten()
        .map(|((id, n), p)| ((id.clone(), n.to_string()), p.to_string()));
    let rids = ids
        .iter()
        .map(|id| {
            vec![
                ((id.clone(), "5"), 1),
                ((id.clone(), "6"), 2),
                ((id.clone(), "7"), 3),
                ((id.clone(), "8"), 4),
            ]
        })
        .flatten()
        .map(|((id, n), u)| ((id, n.to_string()), u as usize));

    let validation_ans: HashMap<(ModelID, String), String> = HashMap::from_iter(validation);
    let prediction_rids: HashMap<(ModelID, String), usize> = HashMap::from_iter(rids);

    let predictions = vec![
        "1,1\n2,2\n3,3\n4,4\n5,5\n6,6\n7,7\n8,8",
        "1,0\n2,0\n3,0\n4,0\n5,0\n6,0\n7,0\n8,0",
        "1,1\n2,2\n3,0\n4,0\n5,0\n6,0\n7,0\n8,0",
    ];

    // Tests for classification-based problems

    let info = ClusterInfo {
        project_id: ObjectId::with_string(common::USER_ID).unwrap(),
        columns: HashMap::new(),
        config: ClientMessage::JobConfig {
            timeout: 0,
            column_types: vec![],
            prediction_column: "".to_string(),
            prediction_type: PredictionType::Classification,
        },
        validation_ans: validation_ans.clone(),
        prediction_rids: prediction_rids.clone(),
    };

    let mut model_predictions: HashMap<(ModelID, usize), String> = HashMap::new();
    let mut model_errors: HashMap<ModelID, Option<f64>> = HashMap::new();

    for (model, prediction) in ids.iter().zip(predictions.iter()) {
        let (test, model_error) = evaluate_model(&model, &prediction.to_string(), &info).unwrap();
        for (index, prediction) in test.into_iter() {
            model_predictions.insert((model.clone(), index), prediction);
        }
        model_errors.insert(model.to_string(), Some(model_error));
    }

    let (weights, final_predictions) = weight_predictions(&model_predictions, &model_errors, &info);

    let sum = weights.values().sum::<f64>();
    assert!(approx_eq!(f64, sum, 1.0, ulps = 2));
    assert_eq!(final_predictions.join("\n"), "5\n6\n7\n8");

    // Tests for regression-based problems

    let info = ClusterInfo {
        project_id: ObjectId::with_string(common::USER_ID).unwrap(),
        columns: HashMap::new(),
        config: ClientMessage::JobConfig {
            timeout: 0,
            column_types: vec![],
            prediction_column: "".to_string(),
            prediction_type: PredictionType::Regression,
        },
        validation_ans: validation_ans,
        prediction_rids: prediction_rids,
    };

    let mut model_predictions: HashMap<(ModelID, usize), String> = HashMap::new();
    let mut model_errors: HashMap<ModelID, Option<f64>> = HashMap::new();

    for (model, prediction) in ids.iter().zip(predictions.iter()) {
        let (test, model_error) = evaluate_model(&model, &prediction.to_string(), &info).unwrap();
        for (index, prediction) in test.into_iter() {
            model_predictions.insert((model.clone(), index), prediction);
        }
        model_errors.insert(model.to_string(), Some(model_error));
    }

    let (weights, final_predictions) = weight_predictions(&model_predictions, &model_errors, &info);
    let sum = weights.values().sum::<f64>();
    assert!(approx_eq!(f64, sum, 1.0, ulps = 2));
    for (prediction, actual) in final_predictions.iter().zip("5\n6\n7\n8".split("\n")) {
        assert!(prediction.parse::<f64>().unwrap() - actual.parse::<f64>().unwrap() < 0.1);
    }
}

#[tokio::test]
async fn test_reimbuse_client() {
    let (database, _) = common::initialise_with_db().await;
    let database = Arc::new(database);
    let pricing = Pricing::new(10.0, 0.1);
    let weight = 10.0;
    pricing
        .reimburse(
            database.clone(),
            ObjectId::with_string(common::USER_ID).unwrap(),
            weight,
        )
        .await
        .unwrap();

    let users = database.collection("users");

    let filter = doc! { "_id": ObjectId::with_string(common::USER_ID).unwrap() };
    let user_doc = users.find_one(filter.clone(), None).await.unwrap().unwrap();

    let user: User = mongodb::bson::de::from_document(user_doc).unwrap();

    let amount: i32 = (((&pricing.revenue - (&pricing.revenue * &pricing.commision_rate)) * weight)
        * 100.0) as i32;

    assert_eq!(user.credits, amount);
}

#[tokio::test]
async fn test_model_performance() {
    let (database, _) = common::initialise_with_db().await;

    let model_weights: HashMap<ModelID, f64> = [
        (String::from(common::MODEL1_ID), 0.45),
        (String::from(common::MODEL2_ID), 0.55),
    ]
    .iter()
    .cloned()
    .collect();

    let bad_models: Vec<ModelID> = vec![String::from(common::MODEL3_ID)];

    let database = Arc::new(database);
    let proj_id = ObjectId::with_string(common::PROJECT_ID).unwrap();

    model_performance(Arc::clone(&database), model_weights.clone(), &proj_id, None)
        .await
        .unwrap();

    penalise(Arc::clone(&database), bad_models, &proj_id, None)
        .await
        .unwrap();

    // working out what they should be
    let mut job_perf_vec: Vec<f64> = Vec::new();
    for (_, weight) in model_weights.iter() {
        let val = (weight * (model_weights.len()) as f64) - 1.0;
        let perf: f64 = 0.5 * ((2.0 * val).tanh()) + 0.5;

        job_perf_vec.push(perf);
    }

    job_perf_vec.push(0.0);

    let job_perfs = database.collection("job_performances");
    let filter = doc! {"project_id": ObjectId::with_string(common::PROJECT_ID).unwrap()};
    let mut cursor = job_perfs.find(filter, None).await.unwrap();

    let mut db_vec: Vec<f64> = Vec::new();
    while let Some(doc) = cursor.next().await {
        db_vec.push(doc.unwrap().get_f64("performance").unwrap());
    }

    for res in job_perf_vec.iter() {
        let mut flag: bool = false;
        for val in db_vec.iter() {
            if approx_eq!(f64, *res, *val, ulps = 2) {
                flag = true;
            }
        }
        assert!(flag);
    }

    assert!(approx_eq!(
        f64,
        job_perf_vec.iter().sum(),
        db_vec.iter().sum(),
        ulps = 2
    ));
}
