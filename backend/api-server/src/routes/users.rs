//! Defines the routes specific to user operations.

use ammonia::clean_text;
use async_std::stream::StreamExt;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::Request;

use crate::routes::response_from_json;
use crate::State;
use crypto::generate_user_api_key;
use models::users::User;

/// Gets a user given their database identifier.
///
/// Given a user identifier, finds the user in the database and returns them as a JSON object. If
/// the user does not exist, the handler will panic.
pub async fn get(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let users = database.collection("users");

    let id: String = req.param("user_id")?;
    let object_id = ObjectId::with_string(&id).unwrap();

    let filter = doc! { "_id": object_id };
    // TODO: This can fail if the user does not exist
    let document = users.find_one(filter, None).await?.unwrap();

    let json: User = mongodb::bson::de::from_document(document).unwrap();
    Ok(response_from_json(json))
}

/// Gets all users who match a filter.
///
/// Given a filter query, finds all users who match the filter and returns them as a JSON array of
/// objects. For example, given `{"first_name", "John"}`, finds all the users with the first name
/// John.
pub async fn filter(mut req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let users = database.collection("users");
    let filter: Document = req.body_json().await?;

    println!("Filter: {:?}", &filter);

    let cursor = users.find(filter, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    Ok(response_from_json(documents.unwrap()))
}

/// Creates a new user given the form information.
///
/// Given an email, password, first name and last name, peppers their password and hashes it. This
/// then gets stored in the Mongo database with a randomly generated user identifier. If the user's
/// email already exists, the route will not register any user.
/// The user's client status will be false which can be later changed
pub async fn new(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    log::debug!("Document received: {:?}", &doc);

    let state = req.state();
    let pepper = &state.pepper;

    let database = state.client.database("sybl");
    let users = database.collection("users");

    let password = doc.get_str("password").unwrap();
    let email = clean_text(doc.get_str("email").unwrap());
    let first_name = clean_text(doc.get_str("firstName").unwrap());
    let last_name = clean_text(doc.get_str("lastName").unwrap());

    log::info!("Email: {}, Password: {}", email, password);
    log::info!("Name: {} {}", first_name, last_name);

    let filter = doc! { "email": &email };

    log::info!("Checking if the user exists already");

    if users.find_one(filter, None).await?.is_some() {
        log::error!("Found a user with email '{}' already", &email);
        return Ok(response_from_json(doc! {"token": "null"}));
    }

    log::info!("User does not exist, registering them now");

    let peppered = format!("{}{}", &password, &pepper);
    let pbkdf2_hash = pbkdf2::pbkdf2_simple(&peppered, state.pbkdf2_iterations).unwrap();

    log::info!("Hash: {:?}", pbkdf2_hash);

    // Generate an API key for the user
    let api_key = generate_user_api_key();

    let credits = 10;

    let user = User {
        id: Some(ObjectId::new()),
        email,
        password: pbkdf2_hash,
        first_name,
        last_name,
        api_key,
        client: false,
        credits,
    };

    let document = mongodb::bson::ser::to_document(&user).unwrap();
    let id = users.insert_one(document, None).await?.inserted_id;

    Ok(response_from_json(
        doc! {"token": id.as_object_id().unwrap().to_string()},
    ))
}

/// Edits a user in the database and updates their information.
///
/// Given a user identifier, finds the user in the database and updates their information based on
/// the JSON provided, returning a message based on whether it was updated.
pub async fn edit(mut req: Request<State>) -> tide::Result {
    let state = req.state();
    let database = state.client.database("sybl");
    let users = database.collection("users");

    let doc: Document = req.body_json().await?;
    let object_id = clean_text(doc.get_str("id").unwrap());
    let id = ObjectId::with_string(&object_id).unwrap();

    let filter = doc! {"_id": id};
    let mut user = match users.find_one(filter.clone(), None).await? {
        Some(u) => mongodb::bson::de::from_document::<User>(u).unwrap(),
        None => return Ok(response_from_json(doc! {"status": "failed"})),
    };

    for key in doc.keys() {
        println!("{}", key);

        if key == "email" {
            user.email = clean_text(doc.get_str(key).unwrap());
        }
    }

    let document = mongodb::bson::ser::to_document(&user).unwrap();
    users.update_one(filter.clone(), document, None).await?;

    Ok(response_from_json(doc! {"status": "changed"}))
}

/// Verifies a user's password against the one in the database.
///
/// Given an email and password, finds the user in the database and checks that the two hashes
/// match. If they don't, or the user does not exist, it will not authenticate them and send back a
/// null token.
pub async fn login(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;

    let state = req.state();
    let database = state.client.database("sybl");
    let pepper = &state.pepper;

    let users = database.collection("users");

    let password = doc.get_str("password").unwrap();
    let email = clean_text(doc.get_str("email").unwrap());

    println!("{}, {}", &email, &password);

    let filter = doc! {"email": email};
    let user = users
        .find_one(filter, None)
        .await?
        .map(|doc| mongodb::bson::de::from_document::<User>(doc).unwrap());

    if let Some(user) = user {
        let peppered = format!("{}{}", password, pepper);
        let verified = pbkdf2::pbkdf2_check(&peppered, &user.password).is_ok();

        if verified {
            println!("Logged in: {:?}", user);
            let identifier = user.id.unwrap().to_string();
            Ok(response_from_json(doc! {"token": identifier}))
        } else {
            println!("Failed login: wrong password");
            Ok(response_from_json(doc! {"token": "null"}))
        }
    } else {
        println!("Failed login: wrong email");
        Ok(response_from_json(doc! {"token": "null"}))
    }
}

/// Deletes a user from the database.
///
/// Given a user identifier, deletes the related user from the database if they exist.
pub async fn delete(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;

    let state = &req.state();
    let database = &state.client.database("sybl");
    let users = database.collection("users");

    let object_id = clean_text(doc.get_str("id").unwrap());
    let id = ObjectId::with_string(&object_id).unwrap();
    let filter = doc! {"_id": id};

    users.find_one_and_delete(filter, None).await.unwrap();

    Ok(response_from_json(doc! {"status": "deleted"}))
}
