use crypto::*;
use rsa::hash::Hash::SHA2_256;
use rsa::{PaddingScheme, RSAPrivateKey, RSAPublicKey};

#[test]
fn key_generation() {
    let (private, public) = generate_key_pair();
    assert_eq!(private.to_public_key(), public);
}

#[test]
fn key_encoding() {
    let (prv_str, pub_str) = encoded_key_pair();
    let private = RSAPrivateKey::from_pkcs1(&prepare_pkcs1(prv_str).unwrap())
        .expect("Unable to parse private key");
    let public = RSAPublicKey::from_pkcs1(&prepare_pkcs1(pub_str).unwrap())
        .expect("Unable to parse public key");
    assert_eq!(private.to_public_key(), public);
}

#[test]
fn challenge_response_protocol() {
    let (private, public) = generate_key_pair();
    let challenge = generate_challenge();
    let response = private
        .sign(PaddingScheme::new_pkcs1v15_sign(Some(SHA2_256)), &challenge)
        .expect("Unable to decrypt challenge");
    assert!(verify_challenge(challenge, response, public));
}
