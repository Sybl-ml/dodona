use tide::http::mime;
use tide::{Request, Response};

use crate::State;

pub mod projects;
pub mod users;

pub async fn index(_req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(Response::builder(200)
        .body(json!({"name": "Freddie", "age": 22}))
        .content_type(mime::JSON))
}

pub async fn hello(_req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok("Hey from Dodona!")
}
