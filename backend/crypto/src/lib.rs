//! Contains cryptographic functions used by the web server

use ammonia::clean_text;
use html_escape::decode_html_entities;
use openssl::hash::MessageDigest as MD;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::sign::Verifier;
use pbkdf2::{
    password_hash::{
        HasherError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, VerifyError,
    },
    Params, Pbkdf2,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rand_core::OsRng;
use serde_json::Value;

const KEY_SIZE: u32 = 1024;
const CHALLENGE_SIZE: usize = 32;
const API_KEY_SIZE: usize = 32;
const ACCESS_TOKEN_SIZE: usize = 32;

/// Hashes a user's password.
pub fn hash_password(peppered: &str, rounds: u32) -> Result<String, HasherError> {
    let params = Params {
        rounds,
        ..Default::default()
    };

    let salt = SaltString::generate(&mut OsRng);

    Ok(Pbkdf2
        .hash_password(peppered.as_bytes(), None, None, params, salt.as_salt())?
        .to_string())
}

/// Checks a user's password is correct.
pub fn verify_password(peppered: &str, expected_hash: &str) -> Result<(), VerifyError> {
    let hash = PasswordHash::new(&expected_hash).unwrap();
    Pbkdf2.verify_password(&peppered.as_bytes(), &hash)
}

/// Returns a new, random RSA key pair
///
/// Generates a cryptographically secure random RSA private key of size `KEY_SIZE`,
/// calculates its corresponding RSA public key and returns both values
pub fn generate_key_pair() -> Rsa<Private> {
    Rsa::generate(KEY_SIZE).expect("Unable to generate private key")
}

/// Returns a PKCS1-encoded RSA key pair
///
/// Generates a new RSA key pair and encodes key values into a tuple
/// of strings using PKCS1 encoding
pub fn encoded_key_pair() -> (String, String) {
    let rsa = generate_key_pair();
    (
        String::from_utf8(
            rsa.private_key_to_pem()
                .expect("Unable to PKCS1-encode private key"),
        )
        .unwrap(),
        String::from_utf8(
            rsa.public_key_to_pem()
                .expect("Unable to PKCS1-encode public key"),
        )
        .unwrap(),
    )
}

/// Returns a random string of size `n`
///
/// Given a size `n`, generates a sequence of `n` cryptographically
/// secure random characters and returns this as a string
pub fn generate_string(n: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric) as char)
        .take(n)
        .collect()
}

/// Returns a binary challenge for use in cryptographic protocols
///
/// Generates a cryptographically secure random binary vector of
/// length `CHALLENGE_SIZE` from alphanumeric characters
pub fn generate_challenge() -> Vec<u8> {
    generate_string(CHALLENGE_SIZE).as_bytes().to_vec()
}

/// Returns true if and only if a challenge is correctly verified
///
/// Given a `challenge`, a `response` and an associated `public_key`,
/// verify the `challenge` by decrypting the `response` using the `public_key`
/// and asserting that the `challenge` and the decrypted `response` match
pub fn verify_challenge(challenge: &[u8], response: &[u8], public_key: &str) -> bool {
    let rsa = Rsa::public_key_from_pem(public_key.as_bytes()).expect("Unable to parse public key");
    let keypair = PKey::from_rsa(rsa).unwrap();
    let mut verifier = Verifier::new(MD::sha256(), &keypair).unwrap();
    verifier.verify_oneshot(&response, &challenge).unwrap()
}

/// Generates a user API key of `API_KEY_SIZE` alphanumeric characters.
pub fn generate_user_api_key() -> String {
    generate_string(API_KEY_SIZE)
}

/// Generates a model access token of `ACCESS_TOKEN_SIZE` alphanumeric characters.
pub fn generate_access_token() -> Vec<u8> {
    generate_string(ACCESS_TOKEN_SIZE).as_bytes().to_vec()
}

/// Sanitises user input to mitigate against XSS attacks, etc.
///
/// Cleans `s` by removing any unauthorised elements, such as `<script>`
/// tags, then returns the HTML-decoded value of the sanitised string
/// to ensure that valid characters, such as ' ' or '\n', are preserved
pub fn clean(s: &str) -> String {
    decode_html_entities(&clean_text(&s)).to_string()
}

/// Sanitises a JSON object to mitigate against XSS attacks, etc.
///
/// Recursively sanitises `json` by applying `clean` to any `String` values,
/// and by mapping `clean` across any `Array` or `Object` data structures.
pub fn clean_json(json: Value) -> Value {
    match json {
        Value::String(s) => Value::String(clean(&s)),
        Value::Array(v) => Value::Array(v.into_iter().map(clean_json).collect()),
        Value::Object(m) => Value::Object(m.into_iter().map(|(k, v)| (k, clean_json(v))).collect()),
        _ => json,
    }
}
