#![allow(dead_code)]
use chrono::TimeZone;
use config::Environment;
use mongodb::bson::{self, document::Document, oid::ObjectId, Binary};
use mongodb::Database;
use std::env;
use std::str::FromStr;

use models::models::ClientModel;
use models::projects::Status;
use utils::compress::compress_data;

static MUTEX: tokio::sync::Mutex<bool> = tokio::sync::Mutex::const_new(true);

pub static USER_ID: &str = "5f8ca1a80065f27b0089e8b5";
pub static PROJECT_ID: &str = "5f8ca1a80065f27c0089e8b5";
pub static DATASET_ID: &str = "5f8ca1a80065f27b0089e8b6";
pub static MODEL1_ID: &str = "5f8ca1a80065f27b0089e8b7";
pub static MODEL2_ID: &str = "5f8ca1a80065f27b0089e8b8";
pub static MODEL3_ID: &str = "5f8ca1a80065f27b0089e8b9";
pub static DATASET: &str = "col1,col2,\nr1c1,r1c2,\nr2c1,r2c2,\n";

pub struct Params {
    pub conn_str: String,
    pub node_socket: u16,
    pub broker_socket: u16,
    pub database_name: String,
}

pub fn initialise() -> Params {
    let config = config::ConfigFile::from_filesystem();
    let resolved = config.resolve(Environment::Testing);
    resolved.populate_environment();
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let node_socket =
        u16::from_str(&env::var("NODE_SOCKET").expect("NODE_SOCKET must be set")).unwrap();
    let broker_socket =
        u16::from_str(&env::var("BROKER_PORT").unwrap_or_else(|_| "9092".to_string())).unwrap();
    let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| String::from("sybl"));

    Params {
        conn_str,
        node_socket,
        broker_socket,
        database_name,
    }
}

pub async fn initialise_with_db() -> (Database, Params) {
    // Acquire the mutex
    let mut lock = MUTEX.lock().await;

    let params = initialise();

    // Ensure that we aren't using the Atlas instance
    assert!(
        !params.conn_str.starts_with("mongodb+srv"),
        "Please setup a local MongoDB instance for running the tests"
    );

    let client = mongodb::Client::with_uri_str(&params.conn_str)
        .await
        .unwrap();
    let database = client.database(&params.database_name);

    // Check whether this is the first time being run
    if *lock {
        let collection_names = database.list_collection_names(None).await.unwrap();

        // Delete all records currently in the database
        for name in collection_names {
            let collection = database.collection(&name);
            collection.delete_many(Document::new(), None).await.unwrap();
        }

        let peppered = format!("password{}", std::env::var("PEPPER").unwrap());
        let pbkdf2_iterations =
            u32::from_str(&std::env::var("PBKDF2_ITERATIONS").unwrap()).unwrap();
        let hash = crypto::hash_password(&peppered, pbkdf2_iterations).unwrap();

        let matthew = bson::doc! {
            "_id": ObjectId::with_string(USER_ID).unwrap(),
            "email": "matthewsmith@email.com",
            "hash": hash,
            "first_name": "Matthew",
            "last_name": "Smith",
            "api_key": "",
            "client": false,
            "credits" : 0,
        };

        let users = database.collection("users");
        users.insert_one(matthew, None).await.unwrap();

        let project = bson::doc! {
            "_id": ObjectId::with_string(PROJECT_ID).unwrap(),
            "name": "Test Project",
            "description": "Test Description",
            "date_created": bson::Bson::DateTime(chrono::Utc.timestamp_millis(0)),
            "user_id": ObjectId::with_string(USER_ID).unwrap(),
            "status": Status::Ready,
        };

        let projects = database.collection("projects");
        projects.insert_one(project, None).await.unwrap();

        let dataset = bson::doc! {
            "_id": ObjectId::with_string(DATASET_ID).unwrap(),
            "project_id": ObjectId::with_string(PROJECT_ID).unwrap(),
            "dataset": Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: compress_data(DATASET).unwrap(),
            },
            "predict": Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: compress_data(DATASET).unwrap(),
            },
        };

        let datasets = database.collection("datasets");
        datasets.insert_one(dataset, None).await.unwrap();

        let mut model1 = ClientModel::new(
            ObjectId::with_string(USER_ID).unwrap(),
            String::from("Model1"),
            Vec::new(),
        );

        model1.id = ObjectId::with_string(MODEL1_ID).unwrap();

        let model2 = ClientModel::new(
            ObjectId::with_string(USER_ID).unwrap(),
            String::from("Model2"),
            Vec::new(),
        );

        let models = database.collection("models");
        models
            .insert_many(
                vec![
                    bson::ser::to_document(&model1).unwrap(),
                    bson::ser::to_document(&model2).unwrap(),
                ],
                None,
            )
            .await
            .unwrap();

        // Update the lock
        *lock = false;
    }
    return (database, params);
}
