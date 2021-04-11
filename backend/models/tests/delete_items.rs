use mongodb::bson::{doc, oid::ObjectId};
use mongodb::error::Result;

use models::datasets::Dataset;
use models::models::ClientModel;
use models::predictions::Prediction;
use models::projects::Project;
use models::users::{Client, User};

mod common;

#[tokio::test]
async fn user_tree_can_be_deleted() -> mongodb::error::Result<()> {
    let (db, _lock) = common::initialise().await;
    let users = db.collection("users");
    let projects = db.collection("projects");
    let datasets = db.collection("datasets");
    let details = db.collection("details");
    let predictions = db.collection("predictions");

    let uid = ObjectId::with_string(common::USER_ID).unwrap();
    let filter = doc! {"_id": uid};
    let user_doc = users.find_one(filter.clone(), None).await?.unwrap();
    let user: User = mongodb::bson::de::from_document(user_doc).unwrap();

    user.delete(&db).await?;

    assert!(users.find_one(filter, None).await?.is_none());
    assert!(projects.find_one(None, None).await?.is_none());
    assert!(datasets.find_one(None, None).await?.is_none());
    assert!(details.find_one(None, None).await?.is_none());
    assert!(predictions.find_one(None, None).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn client_users_can_be_deleted() -> Result<()> {
    let (db, _lock) = common::initialise().await;
    let users = db.collection("users");
    let clients = db.collection("clients");
    let models = db.collection("models");

    let cid = ObjectId::with_string(common::CLIENT_USER_ID).unwrap();
    let filter = doc! {"_id": cid};
    let client_doc = users.find_one(filter.clone(), None).await?.unwrap();
    let user: User = mongodb::bson::de::from_document(client_doc).unwrap();

    user.delete(&db).await?;

    assert!(users.find_one(filter, None).await?.is_none());
    assert!(clients.find_one(None, None).await?.is_none());
    assert!(models.find_one(None, None).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn project_tree_can_be_deleted() -> Result<()> {
    let (db, _lock) = common::initialise().await;
    let projects = db.collection("projects");
    let datasets = db.collection("datasets");
    let details = db.collection("details");
    let predictions = db.collection("predictions");

    let pid = ObjectId::with_string(common::PROJECT_ID).unwrap();
    let filter = doc! {"_id": &pid};
    let proj_doc = projects.find_one(filter.clone(), None).await?.unwrap();
    let project: Project = mongodb::bson::de::from_document(proj_doc).unwrap();
    project.delete(&db).await?;
    let pid_filter = doc! {"project_id": &pid};

    assert!(projects.find_one(filter, None).await?.is_none());
    assert!(datasets.find_one(pid_filter.clone(), None).await?.is_none());
    assert!(details.find_one(pid_filter.clone(), None).await?.is_none());
    assert!(predictions.find_one(pid_filter, None).await?.is_none());

    let pid_2 = ObjectId::with_string(common::PROJECT_ID_2).unwrap();
    assert!(projects
        .find_one(doc! {"_id": &pid_2}, None)
        .await?
        .is_some());
    Ok(())
}

#[tokio::test]
async fn predict_can_be_deleted() -> Result<()> {
    let (db, _lock) = common::initialise().await;
    let predictions = db.collection("predictions");

    let pred_id = ObjectId::with_string(common::PREDICTION_ID).unwrap();
    let filter = doc! {"_id": &pred_id};
    let pred_doc = predictions.find_one(filter.clone(), None).await?.unwrap();
    let prediction: Prediction = mongodb::bson::de::from_document(pred_doc).unwrap();
    prediction.delete(&db).await?;

    assert!(predictions
        .find_one(doc! {"_id": &pred_id}, None)
        .await?
        .is_none());
    Ok(())
}

#[tokio::test]
async fn dataset_can_be_deleted() -> Result<()> {
    let (db, _lock) = common::initialise().await;
    let datasets = db.collection("datasets");
    let details = db.collection("details");

    let id = ObjectId::with_string(common::DATASET_ID).unwrap();
    let pid = ObjectId::with_string(common::PROJECT_ID).unwrap();

    let filter = doc! {"_id": &id};
    let data_doc = datasets.find_one(filter.clone(), None).await?.unwrap();
    let dataset: Dataset = mongodb::bson::de::from_document(data_doc).unwrap();
    dataset.delete(&db).await?;

    assert!(datasets.find_one(filter, None).await?.is_none());
    assert!(details
        .find_one(doc! {"project_id": pid}, None)
        .await?
        .is_none());
    Ok(())
}

#[tokio::test]
async fn models_can_be_deleted() -> Result<()> {
    let (db, _lock) = common::initialise().await;
    let models = db.collection("models");

    let model_id = ObjectId::with_string(common::MODEL_ID).unwrap();
    let filter = doc! {"_id": &model_id};
    let model_doc = models.find_one(filter.clone(), None).await?.unwrap();
    let model: ClientModel = mongodb::bson::de::from_document(model_doc).unwrap();

    model.delete(&db).await?;

    assert!(models.find_one(filter, None).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn client_tree_can_be_deleted() -> Result<()> {
    let (db, _lock) = common::initialise().await;
    let clients = db.collection("clients");
    let models = db.collection("models");

    let cid = ObjectId::with_string(common::CLIENT_USER_ID).unwrap();
    let filter = doc! {"user_id": cid};
    let client_doc = clients.find_one(filter.clone(), None).await?.unwrap();
    let client: Client = mongodb::bson::de::from_document(client_doc).unwrap();

    client.delete(&db).await?;

    assert!(clients.find_one(filter, None).await?.is_none());
    assert!(models.find_one(None, None).await?.is_none());

    Ok(())
}
