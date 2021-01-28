//! Defines the structure of users in the `MongoDB` instance.
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::projects::Project;

/// Defines the information that should be stored with a user in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// The unique identifier for the user
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The user's email
    pub email: String,
    /// The peppered and hashed version of the user's password
    pub hash: String,
    /// The user's first name
    pub first_name: String,
    /// The user's last name
    pub last_name: String,
    /// The user's API key
    pub api_key: String,
    /// Whether the user is a client or not
    pub client: bool,
    /// The user's credits
    pub credits: i32,
}

impl User {
    /// Creates a new instance of [`User`].
    pub fn new<T: Into<String>>(email: T, hash: T, first_name: T, last_name: T) -> Self {
        Self {
            id: ObjectId::new(),
            email: email.into(),
            hash: hash.into(),
            first_name: first_name.into(),
            last_name: last_name.into(),
            api_key: crypto::generate_user_api_key(),
            client: false,
            credits: 10,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let users = database.collection("users");
        let projects = database.collection("projects");
        let clients = database.collection("clients");

        let filter = doc! { "_id": &self.id };
        // Remove user from database
        users.delete_one(filter, None).await?;

        // Remove users projects from the database
        let project_filter = doc! { "user_id": &self.id};
        let mut cursor = projects.find(project_filter, None).await?;

        while let Some(Ok(project_doc)) = cursor.next().await {
            let proj: Project = mongodb::bson::de::from_document(project_doc).unwrap();
            proj.delete(database).await?;
        }

        let client_filter = doc! { "user_id": &self.id};
        let client = clients.find_one(client_filter, None).await?;

        if let Some(client) = client {
            let client: Client = mongodb::bson::de::from_document(client).unwrap();
            client.delete(&database).await?;
        }

        Ok(())
    }
}

/// Defines the information that should be stored with a client in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    /// The unique identifier for the client
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The identifier of the user to which this client information belongs
    pub user_id: ObjectId,
    /// This clients public Key
    pub public_key: String,
}

impl Client {
    pub fn new(user_id: ObjectId, public_key: String) -> Self {
        Self {
            id: ObjectId::new(),
            user_id,
            public_key,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let clients = database.collection("clients");
        let models = database.collection("models");

        // might need to be self.user_id
        let filter = doc! {"_id": &self.id};
        clients.delete_one(filter, None).await?;

        let model_filter = doc! {"user_id": &self.user_id};
        let mut cursor = models.find(model_filter, None).await?;

        while let Some(Ok(model_doc)) = cursor.next().await {
            let model: Project = mongodb::bson::de::from_document(model_doc).unwrap();
            model.delete(database).await?;
        }

        Ok(())
    }
}
