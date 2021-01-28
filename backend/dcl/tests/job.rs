use std::collections::HashMap;
use std::time::Duration;

use dcl::job_end::{ModelID, WriteBackMemory};

mod common;

pub static MODEL_ID: ModelID = 10;

#[tokio::test]
async fn test_write_back_predictions() {
    let id1 = String::from("rd1");
    let val1 = String::from("pred1");

    let mut pred_map = HashMap::new();
    pred_map.insert(id1.clone(), val1.clone());

    let wb: WriteBackMemory = WriteBackMemory::new();

    let wb_clone = wb.clone();
    let pred_map_clone = pred_map.clone();

    tokio::spawn(async move {
        wb_clone.write_predictions(MODEL_ID, pred_map_clone);
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let predictions = wb.get_predictions();
    let pred_val = predictions.get(&(MODEL_ID, id1.clone())).unwrap();
    assert_eq!(&val1, pred_val);
}

#[tokio::test]
async fn test_write_back_errors() {
    let error: f64 = 10.0;

    let wb: WriteBackMemory = WriteBackMemory::new();

    let wb_clone = wb.clone();

    tokio::spawn(async move {
        wb_clone.write_error(MODEL_ID, error);
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let errors = wb.get_errors();
    let error_val = errors.get(&MODEL_ID).unwrap();
    assert_eq!(&error, error_val);
}
