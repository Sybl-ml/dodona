//! Defines the routes for the API server.

use tide::{http::mime, Response};

pub mod projects;
pub mod users;
pub mod jobs;

/// Builds a [`Response`] with a 200 OK and JSON payload.
///
/// Defines a helper function that can be used to quickly build a JSON payload response to a user
/// request. As many API functions take and return JSON, this reduces repetition when the route has
/// been processed correctly.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
/// use dodona::routes::response_from_json;
///
/// #[derive(Serialize)]
/// struct Payload {
///     name: String,
///     age: u32,
/// }
///
/// let payload = Payload { name: String::from("Freddie"), age: 22 };
/// let mut response = response_from_json(payload);
///
/// assert_eq!(response.status(), tide::StatusCode::Ok);
/// assert_eq!(response.content_type(), Some(tide::http::mime::JSON));
///
/// let body = async_std::task::block_on(response.take_body().into_string());
/// let expected = String::from(r#"{"name":"Freddie","age":22}"#);
///
/// assert!(body.is_ok());
/// assert_eq!(body.unwrap(), expected);
/// ```
pub fn response_from_json<B: serde::Serialize>(body: B) -> Response {
    Response::builder(200)
        .body(json!(body))
        .content_type(mime::JSON)
        .build()
}
