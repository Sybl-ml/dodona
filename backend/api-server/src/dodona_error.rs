//! Defines error handling for the api_server

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

/// Defines HTTP errors which can be returned
#[derive(Error, Debug)]
pub enum DodonaError {
    /// Not Found HTTP Error
    #[error("Requested file was not found")]
    NotFound,
    /// Forbidden HTTP Error
    #[error("You are forbidden to access requested file.")]
    Forbidden,
    /// Internal Server Error HTTP Error
    #[error("Unknown Internal Error")]
    Unknown,
    /// UNPROCESSABLE_ENTITY HTTP Error
    #[error("Invalid Request")]
    Invalid,
    /// Conflict HTTP Error
    #[error("Conflict")]
    Conflict,
    /// Unauthorized access HTTP Error
    #[error("Unauthorized")]
    Unauthorized,
}

impl DodonaError {
    /// Method to return string version of Enum
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::Invalid => "Invalid".to_string(),
            Self::Conflict => "Conflict".to_string(),
            Self::Unauthorized => "Unauthorized".to_string(),
        }
    }
}

impl ResponseError for DodonaError {
    /// Function to return the HTTP status code of Enum
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Invalid => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    /// Function which builds error response
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

/// Error Response Enum
#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
