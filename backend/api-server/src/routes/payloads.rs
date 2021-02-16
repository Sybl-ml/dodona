//! Contains the expected payloads for each endpoint.

use mongodb::bson::Document;

/// Stores the options for filtering all users.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterUsersOptions {
    /// The filter to apply to the database
    pub filter: Document,
}

/// Stores the options for registering a user.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationOptions {
    /// The first name of the user
    pub first_name: String,
    /// The last name of the user
    pub last_name: String,
    /// The user's email address
    pub email: String,
    /// The user's password
    pub password: String,
}

/// Stores the options for logging in a user.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginOptions {
    /// The user's email address
    pub email: String,
    /// The user's password
    pub password: String,
}

/// Stores the options for editing a user.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditUserOptions {
    /// The new email address for the user
    pub email: String,
}

/// Stores the options for creating a new project.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProjectOptions {
    /// The name of the project
    pub name: String,
    /// The description of the project
    pub description: String,
}

/// Stores the options for uploading a dataset.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadDatasetOptions {
    /// The name of the file being uploaded
    pub name: String,
    /// The content of the file being uploaded
    pub content: String,
}

/// Stores the options for patching a project.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchProjectOptions {
    /// The document to apply to the database
    pub changes: Document,
}

/// Stores the options for beginning processing of a dataset.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingOptions {
    /// The timeout for the job
    pub timeout: u32,
    /// The type of prediction category this is
    pub prediction_type: String,
    /// The column to use for prediction
    pub prediction_column: String,
}

/// Stores the options for registering a new client.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterClientOptions {
    /// The user's email address
    pub email: String,
    /// The user's password
    pub password: String,
}

/// Stores the options for creating a new model.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewModelOptions {
    /// The user's email address
    pub email: String,
    /// The name of the new model
    pub model_name: String,
}

/// Stores the options for verifying a challenge.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyChallengeOptions {
    /// The user's email address
    pub email: String,
    /// The name of the model
    pub model_name: String,
    /// Their response to the challenge provided
    pub challenge_response: String,
}

/// Stores the options for unlocking a model.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockModelOptions {
    /// The user's password
    pub password: String,
}

/// Stores the options for authenticating a model.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateModelOptions {
    /// The identifier of the model to authenticate
    pub id: String,
    /// The user's access token
    pub token: String,
}
