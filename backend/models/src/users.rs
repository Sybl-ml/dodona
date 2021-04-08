//! Defines the structure of users in the `MongoDB` instance.
use mongodb::bson::{doc, oid::ObjectId, Binary};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::models::ClientModel;
use crate::projects::Project;

pub const STARTING_CREDITS: i32 = 10000;

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
    /// Avatar Image
    pub avatar: Option<Binary>,
}

impl User {
    /// Creates a new instance of [`User`].
    pub fn new<T: Into<String>>(email: T, hash: T, first_name: T, last_name: T) -> Self {
        let email = email.into();
        let hash = hash.into();
        let first_name = first_name.into();
        let last_name = last_name.into();

        log::debug!(
            "Creating a new user with email={}, first_name={}, last_name={}",
            email,
            first_name,
            last_name
        );

        Self {
            id: ObjectId::new(),
            email,
            hash,
            first_name,
            last_name,
            api_key: crypto::generate_user_api_key(),
            client: false,
            credits: STARTING_CREDITS,
            avatar: None,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let users = database.collection("users");
        let projects = database.collection("projects");
        let clients = database.collection("clients");

        log::debug!("Deleting user with id={}, email={}", self.id, self.email);

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
        log::debug!("Creating a new client with user_id={}", user_id);

        Self {
            id: ObjectId::new(),
            user_id,
            public_key,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let clients = database.collection("clients");
        let models = database.collection("models");

        log::debug!("Deleting client with id={}", self.id);

        let filter = doc! {"_id": &self.id};
        clients.delete_one(filter, None).await?;

        let model_filter = doc! {"user_id": &self.user_id};
        let mut cursor = models.find(model_filter, None).await?;

        while let Some(Ok(model)) = cursor.next().await {
            let model: ClientModel = mongodb::bson::de::from_document(model).unwrap();
            model.delete(database).await?;
        }

        Ok(())
    }
}
