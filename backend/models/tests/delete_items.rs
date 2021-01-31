use mongodb::bson::{doc, document::Document, oid::ObjectId};
use mongodb::error::Result;

use models::dataset_details::DatasetDetails;
use models::datasets::Dataset;
use models::models::ClientModel;
use models::predictions::Prediction;
use models::projects::Project;
use models::users::{Client, User};

mod common;

#[tokio::test]
async fn user_can_be_deleted() -> mongodb::error::Result<()> {
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

    assert!(
        users.find_one(filter, None).await?.is_none()
            && projects.find_one(None, None).await?.is_none()
            && datasets.find_one(None, None).await?.is_none()
            && details.find_one(None, None).await?.is_none()
            && predictions.find_one(None, None).await?.is_none()
    );
    Ok(())
}

#[tokio::test]
async fn client_users_can_be_deleted() -> Result<()> {
    let (db, _lock) = common::initialise().await;
    let users = db.collection("users");
    let clients = db.collection("clients");
    let models = db.collection("models");

    let cid = ObjectId::with_string(common::CLIENT_ID).unwrap();
    let filter = doc! {"_id": cid};
    let client_doc = users.find_one(filter.clone(), None).await?.unwrap();
    let user: User = mongodb::bson::de::from_document(client_doc).unwrap();

    user.delete(&db).await?;

    assert!(
        users.find_one(filter, None).await?.is_none()
            && clients.find_one(None, None).await?.is_none()
    );
    assert!(models.find_one(None, None).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn projects_can_be_deleted() -> Result<()> {
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

    assert!(
        projects.find_one(filter, None).await?.is_none()
            && datasets.find_one(pid_filter.clone(), None).await?.is_none()
            && details.find_one(pid_filter.clone(), None).await?.is_none()
            && predictions.find_one(pid_filter, None).await?.is_none()
    );

    let pid_2 = ObjectId::with_string(common::PROJECT_ID_2).unwrap();
    assert!(projects
        .find_one(doc! {"_id": &pid_2}, None)
        .await?
        .is_some());
    Ok(())
}
// async fn predic_can_be_deleted() -> Result<()> {
//     Ok(())
// }
// async fn user_can_be_deleted() -> Result<()> {
//     Ok(())
// }
// async fn user_can_be_deleted() -> Result<()> {
//     Ok(())
// }
// async fn user_can_be_deleted() -> Result<()> {
//     Ok(())
// }
// async fn user_can_be_deleted() -> Result<()> {
//     Ok(())
// }
