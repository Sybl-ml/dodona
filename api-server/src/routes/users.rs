use super::*;
use crate::models::model::Model;
use crate::models::users::User;
use ammonia::clean_text;
use async_std::stream::StreamExt;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use ring::{digest, pbkdf2};
use std::num::NonZeroU32;
use std::str;
use tide;
use tide::{Request, Response};

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
type PasswordHash = [u8; CREDENTIAL_LEN];

/// Function to turn a hash output into a string representation
fn hash_to_string(hash: PasswordHash) -> String {
    let mut res = String::from("");
    for i in hash.iter() {
        res.push(*i as char)
    }
    res
}

/// Will turn a string representation of a hash into
/// a byte array representation
fn string_to_hash(string: String) -> PasswordHash {
    let mut res: PasswordHash = [0u8; CREDENTIAL_LEN];
    for (i, c) in string.chars().enumerate() {
        res[i] = c as u8;
    }
    res
}

/// Function which will return a hash of the provided password
/// inlucding the provided salt
fn hash(password: &str, salt: &str) -> PasswordHash {
    let pbkdf2_iterations = NonZeroU32::new(100_000).unwrap();
    let mut to_store: PasswordHash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        PBKDF2_ALG,
        pbkdf2_iterations,
        salt.as_bytes(),
        password.as_bytes(),
        &mut to_store,
    );
    to_store
}

/// This verifies that the password that is given is the correct one
fn verify(password: &str, hash: PasswordHash, salt: &str) -> bool {
    println!("Password: {}, Salt: {}", &password, &salt);
    let pbkdf2_iterations = NonZeroU32::new(100_000).unwrap();
    match pbkdf2::verify(
        PBKDF2_ALG,
        pbkdf2_iterations,
        salt.as_bytes(),
        password.as_bytes(),
        &hash,
    ) {
        Ok(_) => true,
        _ => false,
    }
}

/// This route will take in a user ID in the request and
/// will return the information for that user
pub async fn get(req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let id = req.param::<String>("user_id")?;
    let object_id = ObjectId::with_string(&id).unwrap();
    let filter = doc! { "_id": object_id };
    let doc = User::find_one(db.clone(), filter, None).await?;
    let response = Response::builder(200)
        .body(json!(doc))
        .content_type(mime::JSON)
        .build();

    Ok(response)
}

/// More general version of get. Allows filter to be passed to
/// the find. This will return a JSON object containing multiple
/// users which fulfill the filter.
pub async fn filter(mut req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let filter: Document = req.body_json().await?;
    println!("Filter: {:?}", &filter);
    let mut cursor = User::find(db.clone(), filter, None).await?;
    let mut docs: Vec<User> = Vec::new();
    while let Some(user) = cursor.next().await {
        docs.push(user?);
    }
    Ok(Response::builder(200)
        .body(json!(docs))
        .content_type(mime::JSON)
        .build())
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
    let state = &req.state();
    let db = &state.client.database("sybl");
    let doc: Document = req.body_json().await?;
    let password: &str = &clean_text(doc.get_str("password").unwrap());
    let email: &str = &clean_text(doc.get_str("email").unwrap());
    println!("Email: {}, Password: {}", email, password);

    let filter = doc! {"email": email};
    match User::find_one(db.clone(), filter, None).await? {
        Some(_) => {
            return Ok(Response::builder(200)
                .body(json!(doc! {"token": "null"}))
                .content_type(mime::JSON)
                .build());
        }
        _ => (),
    };

    let salt: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();

    let pbkdf2_hash = hash(password, &salt);

    let verified = verify(&password, pbkdf2_hash, &salt);

    println!("Verified: {}", verified);

    println!("Hash: {:?}", pbkdf2_hash);
    println!("Salt: {}", &salt);

    let mut user: User = User {
        id: Some(ObjectId::new()),
        email: String::from(email),
        password: hash_to_string(pbkdf2_hash),
        salt: salt,
    };

    user.save(db.clone(), None).await?;
    let user_id = user.id().unwrap();
    Ok(Response::builder(200)
        .body(json!(doc! {"token": user_id.to_string()}))
        .content_type(mime::JSON)
        .build())
}

/// Pass a JSON object with the ObjectId for the user
/// which is being edited and the attributes which are being
/// changed. This should look like:
/// {
///     "_id": "TOKEN"
///     "email": "email@email.com",
///     "password": "password"
/// }
///
/// This will return the status of the transaction
pub async fn edit(mut req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let doc: Document = req.body_json().await?;
    let id = ObjectId::with_string(&clean_text(doc.get_str("id").unwrap())).unwrap();
    let filter = doc! {"_id": id};
    let mut user = match User::find_one(db.clone(), filter, None).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(Response::builder(200)
                .body(json!(doc! {"status": "failed"}))
                .content_type(mime::JSON)
                .build())
        }
    }
    .unwrap();

    for key in doc.keys() {
        println!("{}", key);
        if key == "email" {
            user.email = String::from(&clean_text(doc.get_str(key).unwrap()));
        }
    }

    user.save(db.clone(), None).await?;

    Ok(Response::builder(200)
        .body(json!(doc! {"status": "changed"}))
        .content_type(mime::JSON)
        .build())
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
    let state = &req.state();
    let db = &state.client.database("sybl");
    let doc: Document = req.body_json().await?;
    let password: &str = &clean_text(doc.get_str("password").unwrap());
    let email: &str = &clean_text(doc.get_str("email").unwrap());
    println!("{}, {}", &email, &password);
    let filter = doc! {"email": email};
    let user = User::find_one(db.clone(), filter, None).await?;
    match user {
        Some(user) => {
            let hashed_password = string_to_hash(user.password.clone());

            println!("Hashed Password: {:?}", &hashed_password);
            println!("Salt: {}", &user.salt[..]);
            println!("Email: {}", &user.email[..]);

            let verified = verify(&password, hashed_password, &user.salt[..]);

            if verified {
                println!("Logged in: {:?}", user);
                Ok(Response::builder(200)
                    .body(json!(doc! {"token": user.id().unwrap().to_string()}))
                    .content_type(mime::JSON)
                    .build())
            } else {
                println!("Failed login: wrong password");
                Ok(Response::builder(200)
                    .body(json!(doc! {"token": "null"}))
                    .content_type(mime::JSON)
                    .build())
            }
        }
        None => {
            println!("Failed login: wrong email");
            Ok(Response::builder(200)
                .body(json!(doc! {"token": "null"}))
                .content_type(mime::JSON)
                .build())
        }
    }
}

/// Delete method. Pass ID as part of JSON object and the corressponding user
/// will be deleted from the Database.
pub async fn delete(mut req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let doc: Document = req.body_json().await?;
    let id = ObjectId::with_string(&clean_text(doc.get_str("id").unwrap())).unwrap();
    let filter = doc! {"_id": id};
    User::find_one_and_delete(db.clone(), filter, None)
        .await
        .unwrap();

    Ok(Response::builder(200)
        .body(json!(doc! {"status": "deleted"}))
        .content_type(mime::JSON)
        .build())
}
