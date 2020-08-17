use super::*;
use tide::{http::mime, Request, Response};

pub mod users;

pub async fn index(req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(Response::builder(200)
        .body(json!({"name": "Freddie", "age": 22}))
        .content_type(mime::JSON))
}

pub async fn hello(req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok("Hey from Dodona!")
}