use super::*;
use async_std::stream::StreamExt;
use tide::Request;
use tide;
use crate::models::users::User;
use crate::models::model::Model;


pub async fn show(req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");

    let mut cursor = User::find(db.clone(), None, None).await.unwrap();

    while let Some(user) = cursor.next().await {
        println!("{:?}", user?);
    }

    Ok(tide::Redirect::new(format!("/")).into())
}