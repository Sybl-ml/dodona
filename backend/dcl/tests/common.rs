#![allow(dead_code)]
use chrono::TimeZone;
use config::Environment;
use mongodb::bson::{self, document::Document, oid::ObjectId, Binary};
use mongodb::Database;
use std::env;
use std::str::FromStr;

pub static USER_ID: &str = "5f8ca1a80065f27b0089e8b5";
pub static PROJECT_ID: &str = "5f8ca1a80065f27c0089e8b5";
pub static DATASET_ID: &str = "5f8ca1a80065f27b0089e8b6";
pub static DATASET: &str = "col1,col2,\nr1c1,r1c2,\nr2c1,r2c2,\n";

pub struct Params {
    pub conn_str: String,
    pub node_socket: u16,
    pub interface_socket: u16,
}

pub fn initialise() -> Params {
    let config = config::ConfigFile::from_filesystem();
    let resolved = config.resolve(Environment::Testing);
    resolved.populate_environment();
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let node_socket =
        u16::from_str(&env::var("NODE_SOCKET").expect("NODE_SOCKET must be set")).unwrap();
    let interface_socket =
        u16::from_str(&env::var("INTERFACE_SOCKET").expect("INTERFACE_SOCKET must be set"))
            .unwrap();
    Params {
        conn_str,
        node_socket,
        interface_socket,
    }
}

pub async fn initialise_with_db() -> (Database, Params) {
    let params = initialise();
    let client = mongodb::Client::with_uri_str(&params.conn_str)
        .await
        .unwrap();
    let database = client.database("sybl");

    let collection_names = database.list_collection_names(None).await.unwrap();

    // Delete all records currently in the database
    for name in collection_names {
        let collection = database.collection(&name);
        collection.delete_many(Document::new(), None).await.unwrap();
    }

    let peppered = format!("password{}", std::env::var("PEPPER").unwrap());
    let pbkdf2_iterations = u32::from_str(&std::env::var("PBKDF2_ITERATIONS").unwrap()).unwrap();
    let hash = pbkdf2::pbkdf2_simple(&peppered, pbkdf2_iterations).unwrap();

    let matthew = bson::doc! {
        "_id": ObjectId::with_string(USER_ID).unwrap(),
        "email": "matthewsmith@email.com",
        "password": hash,
        "first_name": "Matthew",
        "last_name": "Smith",
        "api_key": "",
        "client": false,
        "credits" : 100,
    };

    let users = database.collection("users");
    users.insert_one(matthew, None).await.unwrap();

    let project = bson::doc! {
        "_id": ObjectId::with_string(PROJECT_ID).unwrap(),
        "name": "Test Project",
        "description": "Test Description",
        "date_created": bson::Bson::DateTime(chrono::Utc.timestamp_millis(0)),
        "user_id": ObjectId::with_string(USER_ID).unwrap(),
        "status": "Ready"
    };

    let projects = database.collection("projects");
    projects.insert_one(project, None).await.unwrap();

    let dataset = bson::doc! {
        "_id": ObjectId::with_string(DATASET_ID).unwrap(),
        "project_id": ObjectId::with_string(PROJECT_ID).unwrap(),
        "dataset": Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: utils::compress_data(DATASET).unwrap(),
        }
    };

    let datasets = database.collection("datasets");
    datasets.insert_one(dataset, None).await.unwrap();
    (database, params)
}
