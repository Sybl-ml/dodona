use std::convert::TryFrom;
use std::time::Duration;

use actix_web::test::TestRequest;
use jsonwebtoken::{EncodingKey, Header};
use mongodb::bson::oid::ObjectId;

use crate::auth::{get_encoding_key, Claims};

#[test]
fn request_without_authorization_header_is_rejected() {
    let request = TestRequest::default().to_http_request();

    assert!(Claims::try_from(&request).is_err());
}

#[test]
fn request_without_valid_string_authorization_header_is_rejected() {
    let request = TestRequest::default()
        .insert_header(("Authorization", "�"))
        .to_http_request();

    assert!(Claims::try_from(&request).is_err());
}

#[test]
fn request_with_invalid_token_is_rejected() {
    let request = TestRequest::default()
        .insert_header(("Authorization", "some valid utf-8"))
        .to_http_request();

    assert!(Claims::try_from(&request).is_err());
}

#[test]
fn request_with_valid_token_but_no_bearer_is_rejected() {
    // Create a new user with a random ObjectId
    let id = ObjectId::new();
    let user = Claims::new(id, u64::MAX);

    // Setup the JWT with the same settings as above
    let header = Header::default();
    let key = get_encoding_key();
    let encoded = jsonwebtoken::encode(&header, &user, &key).unwrap();

    // Build the request with the produced token
    let request = TestRequest::default()
        .insert_header(("Authorization", encoded))
        .to_http_request();

    assert!(Claims::try_from(&request).is_err());
}

#[test]
fn request_with_valid_expired_token_is_rejected() {
    // Create a new user with a random ObjectId
    let id = ObjectId::new();
    let user = Claims::new(id, u64::MIN);

    // Setup the JWT with the same settings as above
    let header = Header::default();
    let key = get_encoding_key();
    let encoded = jsonwebtoken::encode(&header, &user, &key).unwrap();

    // Build the request with the produced token
    let auth_value = format!("Bearer {}", encoded);
    let request = TestRequest::default()
        .insert_header(("Authorization", auth_value))
        .to_http_request();

    assert!(Claims::try_from(&request).is_err());
}

#[test]
fn request_with_valid_token_is_accepted() {
    // Create a new user with a random ObjectId
    let id = ObjectId::new();
    let user = Claims::new(id, u64::MAX);

    // Setup the JWT with the same settings as above
    let header = Header::default();
    let key = get_encoding_key();
    let encoded = jsonwebtoken::encode(&header, &user, &key).unwrap();

    // Build the request with the produced token
    let auth_value = format!("Bearer {}", encoded);
    let request = TestRequest::default()
        .insert_header(("Authorization", auth_value))
        .to_http_request();

    assert!(Claims::try_from(&request).is_ok());
}

#[test]
fn jwt_tokens_can_expire() {
    // Create a new user with a random ObjectId
    let id = ObjectId::new();
    // Set the token to expire almost immediately, we can get 1 request in first
    let token = Claims::create_token_with_duration(id, Duration::default()).unwrap();

    // Build the request with the produced token
    let auth_value = format!("Bearer {}", token);
    let request = TestRequest::default()
        .insert_header(("Authorization", auth_value))
        .to_http_request();

    assert!(Claims::try_from(&request).is_ok());

    // Wait until the token has expired
    std::thread::sleep(Duration::from_secs(1));

    assert!(Claims::try_from(&request).is_err());
}

#[test]
fn tokens_encoded_with_a_different_key_are_rejected() {
    // Create a new user with a random ObjectId
    let id = ObjectId::new();
    let user = Claims::new(id, u64::MAX);

    // Setup the JWT with the same settings as above
    let header = Header::default();
    let key = EncodingKey::from_secret(b"not the same as above");
    let encoded = jsonwebtoken::encode(&header, &user, &key).unwrap();

    // Build the request with the produced token
    let auth_value = format!("Bearer {}", encoded);
    let request = TestRequest::default()
        .insert_header(("Authorization", auth_value))
        .to_http_request();

    assert!(Claims::try_from(&request).is_err());
}
