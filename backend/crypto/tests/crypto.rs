use crypto::*;
use openssl::rsa::{Padding, Rsa};

#[test]
fn key_generation() {
    generate_key_pair();
}

#[test]
fn key_encoding() {
    let (prv_str, pub_str) = encoded_key_pair();
    let private =
        Rsa::private_key_from_pem(prv_str.as_bytes()).expect("Unable to parse private key");
    let public = Rsa::public_key_from_pem(pub_str.as_bytes()).expect("Unable to parse public key");
    assert_eq!(private.private_key_to_pem().unwrap(), prv_str.as_bytes());
    assert_eq!(public.public_key_to_pem().unwrap(), pub_str.as_bytes());
}

#[test]
fn challenge_response_protocol() {
    let rsa = generate_key_pair();
    let challenge = generate_challenge();
    let mut response = vec![0; rsa.size() as usize];
    rsa.private_encrypt(&challenge, &mut response, Padding::PKCS1)
        .expect("Unable to encrypt challenge");
    assert!(verify_challenge(
        challenge,
        response,
        String::from_utf8(rsa.public_key_to_pem().unwrap()).unwrap()
    ));
}
