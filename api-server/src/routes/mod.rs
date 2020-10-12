use tide::http::mime;
use tide::{Request, Response};

use crate::State;

pub mod projects;
pub mod users;
pub mod jobs;

pub fn response_from_json<B: serde::Serialize>(body: B) -> Response {
    Response::builder(200)
        .body(json!(body))
        .content_type(mime::JSON)
        .build()
}

pub async fn index(_req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(Response::builder(200)
        .body(json!({"name": "Freddie", "age": 22}))
        .content_type(mime::JSON))
}

pub async fn hello(_req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok("Hey from Dodona!")
}
