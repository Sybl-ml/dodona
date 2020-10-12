// use ammonia::clean_text;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::http::mime;
use tide::{Request, Response};

use crate::State;

pub async fn new(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    log::debug!("Document received: {:?}", &doc);

    // let state = req.state();

    // let database = state.client.database("sybl");
    // let users = database.collection("users");

    // let password = doc.get_str("password").unwrap();


    Ok(Response::builder(200)
        .body(json!(doc! {"token": "hello"}))
        .content_type(mime::JSON)
        .build())
}