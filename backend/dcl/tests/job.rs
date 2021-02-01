use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use mongodb::bson::{doc, oid::ObjectId};

use dcl::job_end::{finance::Pricing, ModelID, WriteBackMemory};
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

    tokio::time::sleep(Duration::from_secs(1)).await;

    let predictions = wb.get_predictions();
    let pred_val = predictions.get(&(model_id, 1)).unwrap();
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
