use ammonia::clean_text;
use async_std::stream::StreamExt;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::Request;

use crate::models::users::User;
use crate::routes::response_from_json;
use crate::State;

const PBKDF2_ROUNDS: u32 = 100_000;

/// This route will take in a user ID in the request and
/// will return the information for that user
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

/// More general version of get. Allows filter to be passed to
/// the find. This will return a JSON object containing multiple
/// users which fulfill the filter.
pub async fn filter(mut req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let users = database.collection("users");
    let filter: Document = req.body_json().await?;

    println!("Filter: {:?}", &filter);

    let cursor = users.find(filter, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    Ok(response_from_json(documents.unwrap()))
}

/// New route which will allow the frontend to send an email and password
/// which create a new user. This will return the token for the new user.
/// For this, a JSON object must be sent to the route, e.g:
/// {
///     "email": "email@email.com",
///     "password": "password"
/// }
///
/// This will return the user token
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

    let pbkdf2_hash = pbkdf2::pbkdf2_simple(&peppered, PBKDF2_ROUNDS).unwrap();
    let verified = pbkdf2::pbkdf2_check(&peppered, &pbkdf2_hash).is_ok();

    log::info!("Verified: {}", verified);
    log::info!("Hash: {:?}", pbkdf2_hash);

    let user = User {
        id: Some(ObjectId::new()),
        email,
        password: pbkdf2_hash,
        first_name,
        last_name,
    };

    let document = mongodb::bson::ser::to_document(&user).unwrap();
    let id = users.insert_one(document, None).await?.inserted_id;

    Ok(response_from_json(doc! {"token": id}))
}

/// Pass a JSON object with the ObjectId for the user
/// which is being edited and the attributes which are being
/// changed. This should look like:
/// {
///     "id": "TOKEN"
///     "email": "email@email.com",
///     "password": "password"
/// }
///
/// This will return the status of the transaction
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

/// Login route which will allow the frontend to send an email and password
/// which will be checked against the database. If there is a user with those
/// credentials then a token will be returned. Otherwise "null" will be returned
/// For this, a JSON object must be sent to the route
/// {
///     "email": "email@email.com",
///     "password": "password"
/// }
///
/// This will return the user token
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

/// Delete method. Pass ID as part of JSON object and the corressponding user
/// will be deleted from the Database.
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
