//! Contains cryptographic functions used by the web server

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rsa::hash::Hash::SHA2_256;
use rsa::{
    PaddingScheme, PrivateKeyPemEncoding, PublicKey, PublicKeyPemEncoding, RSAPrivateKey,
    RSAPublicKey,
};

const KEY_SIZE: usize = 1024;
const CHALLENGE_SIZE: usize = 32;

/// Returns a new, random RSA key pair
///
/// Generates a cryptographically secure random RSA private key of size `KEY_SIZE`,
/// calculates its corresponding RSA public key and returns both values
pub fn generate_key_pair() -> (RSAPrivateKey, RSAPublicKey) {
    let mut r = thread_rng();
    let private = RSAPrivateKey::new(&mut r, KEY_SIZE).expect("Unable to generate private key");
    let public = RSAPublicKey::from(&private);
    (private, public)
}

/// Returns a PKCS1-encoded RSA key pair
///
/// Generates a new RSA key pair and encodes key values into a tuple
/// of strings using PKCS1 encoding
pub fn encoded_key_pair() -> (String, String) {
    let (private, public) = generate_key_pair();
    (
        private
            .to_pem_pkcs1()
            .expect("Unable to PKCS1-encode private key"),
        public
            .to_pem_pkcs1()
            .expect("Unable to PKCS1-encode public key"),
    )
}

/// Returns an encoded key with PKCS1 padding removed
///
/// Removes PKCS1 padding from `key` so that the output can
/// be parsed as an RSA key
pub fn remove_pkcs1_padding(key: String) -> String {
    key.lines().filter(|l| !l.starts_with("-")).collect()
}

/// Returns a random string of size `n`
///
/// Given a size `n`, generates a sequence of `n` cryptographically
/// secure random characters and returns this as a string
pub fn generate_string(n: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
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
pub fn verify_challenge(challenge: Vec<u8>, response: Vec<u8>, public_key: RSAPublicKey) -> bool {
    public_key
        .verify(
            PaddingScheme::new_pkcs1v15_sign(Some(SHA2_256)),
            &challenge,
            &response,
        )
        .is_ok()
}
