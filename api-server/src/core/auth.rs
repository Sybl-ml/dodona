use std::num::NonZeroU32;

use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use ring::{digest, pbkdf2};

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;

type PasswordHash = [u8; CREDENTIAL_LEN];

/// Function to turn a hash output into a string representation
pub fn hash_to_string(hash: PasswordHash) -> String {
    let mut res = String::from("");
    for i in hash.iter() {
        res.push(*i as char)
    }
    res
}

/// Will turn a string representation of a hash into
/// a byte array representation
pub fn string_to_hash(string: String) -> PasswordHash {
    let mut res: PasswordHash = [0u8; CREDENTIAL_LEN];
    for (i, c) in string.chars().enumerate() {
        res[i] = c as u8;
    }
    res
}

/// Function which will return a hash of the provided password
/// including the provided salt
pub fn hash(password: &str, salt: &str) -> PasswordHash {
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

/// Generates a string of length `length` using
/// randomly generated UTF-8 characters
pub fn generate_chars(length: usize) -> String {
    return thread_rng()
        .sample_iter::<char, Standard>(Standard)
        .take(length)
        .collect();
}

/// This verifies that the password that is given is the correct one
pub fn verify(password: &str, salt: &str, hash: PasswordHash) -> bool {
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