use std::collections::HashMap;
use std::time::Duration;

use dcl::job_end::{ModelID, WriteBackMemory};

mod common;

#[tokio::test]
async fn test_write_back_predictions() {
    let model_id: ModelID = ModelID::from("ModelID1");
    let id1 = String::from("rd1");
    let val1 = String::from("pred1");

    let mut pred_map = HashMap::new();
    pred_map.insert(id1.clone(), val1.clone());

    let wb: WriteBackMemory = WriteBackMemory::new();

    let wb_clone = wb.clone();
    let pred_map_clone = pred_map.clone();
    let mid_clone = model_id.clone();

    tokio::spawn(async move {
        wb_clone.write_predictions(mid_clone, pred_map_clone);
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let predictions = wb.get_predictions();
    let pred_val = predictions.get(&(model_id, id1.clone())).unwrap();
    assert_eq!(&val1, pred_val);
}

#[tokio::test]
async fn test_write_back_errors() {
    let model_id: ModelID = ModelID::from("ModelID1");
    let error: f64 = 10.0;

    let wb: WriteBackMemory = WriteBackMemory::new();

    let wb_clone = wb.clone();

    let mid_clone = model_id.clone();

    tokio::spawn(async move {
        wb_clone.write_error(mid_clone, error);
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let errors = wb.get_errors();
    let error_val = errors.get(&model_id).unwrap();
    assert_eq!(&error, error_val);
}
