//! Defines routes specific to client operations

use actix_web::{http::StatusCode, web};
use mongodb::bson::de::from_document;
use mongodb::bson::ser::to_document;
use mongodb::bson::{self, doc, document::Document, oid::ObjectId, Binary};
use tokio_stream::StreamExt;

use models::job_performance::JobPerformance;
use models::models::{AccessToken, ClientModel};
use models::users::{Client, User};

use crate::{
    auth,
    error::{ServerError, ServerResponse, ServerResult},
    routes::{payloads, response_from_json, response_from_json_with_code},
    State,
};

/// Upgrades a user account to a client account.
///
/// Checks whether the user is already a client before ensuring the email provided is the same as
/// the user's email, along with the password.
pub async fn register(
    claims: auth::Claims,
    state: web::Data<State>,
    payload: web::Json<payloads::RegisterClientOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");
    let clients = state.database.collection("clients");

    let email = crypto::clean(&payload.email);

    let filter = doc! { "_id": &claims.id };
    let document = users
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    let user: User = from_document(document)?;

    if user.client {
        log::warn!("User with id={} is already a client", user.id);
        return Err(ServerError::Conflict);
    }

    let peppered = format!("{}{}", payload.password, &state.pepper);
    pbkdf2::pbkdf2_check(&peppered, &user.hash)?;

    // Entered and stored email and password match
    if email == user.email {
        // generate public and private key pair
        let (private_key, public_key) = crypto::encoded_key_pair();
        // create a new client object
        users
            .update_one(
                doc! { "_id": &claims.id },
                doc! {"$set": {"client": true}},
                None,
            )
            .await?;

        // Update the user to be a client
        let client = Client::new(claims.id, public_key);

        // store client object in db
        let document = to_document(&client)?;
        clients.insert_one(document, None).await?;

        log::debug!("Upgraded user with id={} to a client", user.id);

        // reponse with private key
        response_from_json(doc! {"privKey": private_key})
    } else {
        log::warn!(
            "User's email and provided email didn't match: {} != {}",
            user.email,
            email
        );

        Err(ServerError::Forbidden)
    }
}

/// Registers a new model for a given user.
///
/// Checks whether the given user is a client and that they do not already have a model with the
/// given name, before inserting it into the database and responding with a challenge for the user.
pub async fn new_model(
    state: web::Data<State>,
    payload: web::Json<payloads::NewModelOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");
    let models = state.database.collection("models");

    let email = crypto::clean(&payload.email);

    let filter = doc! { "email": &email };
    let document = users
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    let user: User = from_document(document)?;

    if !user.client {
        return Err(ServerError::Forbidden);
    }

    let filter = doc! { "user_id": &user.id, "name": &payload.model_name };
    if models.find_one(filter, None).await?.is_some() {
        log::warn!(
            "User with id={} already has a model with name={}",
            user.id,
            payload.model_name
        );

        return Err(ServerError::Conflict);
    }

    // Generate challenge
    let challenge = crypto::generate_challenge();
    let client_model = ClientModel::new(user.id, payload.model_name.clone(), challenge.clone());

    // insert model into database
    let document = to_document(&client_model)?;
    models.insert_one(document, None).await?;

    // return challenge
    response_from_json(doc! {
        "Challenge": {
            "challenge": base64::encode(challenge),
        }
    })
}

/// Verifies a challenge response from a client.
///
/// This will fetch the client's public key from the database and use it to check whether the
/// challenge response they provided is a match based on how they signed it. If it succeeds, the
/// model will be set to authenticated and the client will be sent an access token allowing them to
/// use it for computation. Otherwise, they will be sent an error message.
pub async fn verify_challenge(
    state: web::Data<State>,
    payload: web::Json<payloads::VerifyChallengeOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");
    let clients = state.database.collection("clients");
    let models = state.database.collection("models");

    let email = crypto::clean(&payload.email);
    let filter = doc! { "email": &email };

    let user_doc = users
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;
    let user: User = from_document(user_doc)?;

    // get clients public key matching with that users id
    let filter = doc! { "user_id": &user.id };
    let client_doc = clients
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;
    let client: Client = from_document(client_doc)?;

    let filter = doc! { "user_id": &user.id, "name": &payload.model_name };
    let model_doc = models
        .find_one(filter.clone(), None)
        .await?
        .ok_or(ServerError::NotFound)?;
    let mut model: ClientModel = from_document(model_doc)?;

    let public_key = client.public_key;
    let challenge = &model.challenge.ok_or(ServerError::Unauthorized)?.bytes;

    // needs converting to Vec<u8>
    let challenge_response = base64::decode(&payload.challenge_response)?;

    if !crypto::verify_challenge(challenge.to_vec(), challenge_response, public_key) {
        log::warn!("Provided challenge did not match expected");
        return Err(ServerError::Unauthorized);
    }

    let access_token = AccessToken::new();
    model.authenticated = true;
    model.access_token = Some(access_token.clone());
    model.challenge = None;

    let update = doc! { "$set": to_document(&model)? };
    models.find_one_and_update(filter, update, None).await?;

    // return the access token to the model
    response_from_json(doc! {
        "AccessToken": {
            "id": model.id.to_string(),
            "token": base64::encode(access_token.clone().token.bytes),
            "expires": access_token.expires.to_rfc3339()
        }
    })
}

/// Unlocks a given model using multifactor authentication.
///
/// Given the identifier for a model and the user's password, unlocks a model if the password is
/// correct and the model has been authenticated previously. This means it must come after a call
/// to [`verify_challenge`].
pub async fn unlock_model(
    claims: auth::Claims,
    state: web::Data<State>,
    model_id: web::Path<String>,
    payload: web::Json<payloads::UnlockModelOptions>,
) -> ServerResponse {
    let models = state.database.collection("models");
    let users = state.database.collection("users");

    let model_id = ObjectId::with_string(&model_id)?;
    let filter = doc! { "_id": &model_id };
    let model_doc = models
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::Unauthorized)?;
    let mut model: ClientModel = from_document(model_doc)?;

    // Check the current user owns this model
    if model.user_id != claims.id {
        return Err(ServerError::Unauthorized);
    }

    let filter = doc! { "_id": &claims.id };
    let user_doc = users
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::Unauthorized)?;
    let user: User = from_document(user_doc)?;

    let peppered = format!("{}{}", payload.password, &state.pepper);
    let verified = pbkdf2::pbkdf2_check(&peppered, &user.hash).is_err();

    if !model.authenticated || verified {
        log::warn!(
            "model.authenticated: {}, verified: {}",
            model.authenticated,
            verified
        );

        return Err(ServerError::Unauthorized);
    }

    model.locked = false;

    let filter = doc! { "_id": &model_id };
    let update = doc! { "$set": to_document(&model)? };
    models.find_one_and_update(filter, update, None).await?;

    response_from_json(doc! { "message": "Model successfully unlocked" })
}

/// Authenticates a model using its access token.
///
/// Given a model identifier and an access token, ensures that the given token is valid for the
/// provided identifier. This ensures that the model itself is authenticated and unlocked first,
/// before checking whether the token has expired. If it has, the client will be sent a new
/// challenge that they will be expected to complete.
pub async fn authenticate_model(
    state: web::Data<State>,
    model_id: web::Path<String>,
    payload: web::Json<payloads::AuthenticateModelOptions>,
) -> ServerResponse {
    let models = state.database.collection("models");

    let model_id = ObjectId::with_string(&model_id)?;
    let filter = doc! { "_id": &model_id };
    let model_doc = models
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::Unauthorized)?;
    let mut model: ClientModel = from_document(model_doc)?;

    let token = base64::decode(&payload.token)?;

    if !model.is_authenticated(&token) {
        log::warn!(
            "Model failed to authenticate with token={} (base-64)",
            payload.token
        );

        return Err(ServerError::Unauthorized);
    }

    // Check whether their token has expired
    if model.token_has_not_expired() {
        // TODO: authenticate the model in the session
        response_from_json(doc! {"message": "Authentication successful"})
    } else {
        log::warn!("Model with id={} has an expired token", model.id);

        let challenge = crypto::generate_challenge();
        model.authenticated = false;
        model.challenge = Some(Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: challenge.clone(),
        });

        let filter = doc! { "_id": &model_id };
        let update = doc! { "$set": to_document(&model)? };
        models.find_one_and_update(filter, update, None).await?;

        let json = doc! { "challenge": base64::encode(challenge) };

        response_from_json_with_code(json, StatusCode::UNAUTHORIZED)
    }
}

/// Finds all the models related to a given user.
///
/// Given a user identifier, finds all the models in the database that the user owns. If the user
/// doesn't exist or an invalid identifier is given, returns a 404 response.
pub async fn get_user_models(claims: auth::Claims, state: web::Data<State>) -> ServerResponse {
    let models = state.database.collection("models");

    let filter = doc! { "user_id": &claims.id };
    let cursor = models.find(filter, None).await?;
    let documents: Vec<Document> = cursor.collect::<Result<_, _>>().await?;

    response_from_json(documents)
}

/// Gets the model performance for the last 5 jobs.
///
/// Given a model identifier, returns the performance of that model on the last 5 jobs that it
/// completed.
pub async fn get_model_performance(
    state: web::Data<State>,
    model_id: web::Path<String>,
) -> ServerResponse {
    let job_performances = state.database.collection("job_performances");

    let filter = doc! {"model_id": ObjectId::with_string(&model_id)?};

    let build_options = mongodb::options::FindOptions::builder()
        .sort(doc! {"date_created": -1})
        .build();

    let cursor = job_performances.find(filter, Some(build_options)).await?;

    let get_performance = |doc: Document| -> ServerResult<f64> {
        let job_performance: JobPerformance = from_document(doc)?;
        Ok(job_performance.performance)
    };

    let performances: Vec<_> = cursor
        .take(5)
        .filter_map(Result::ok)
        .map(get_performance)
        .collect::<ServerResult<_>>()
        .await?;

    response_from_json(performances)
}
