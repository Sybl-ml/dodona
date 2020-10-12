use crate::core::auth;

#[test]
fn test_sanity() {
    assert!(true);
}

#[test]
fn test_hash_strings() {
    let salt = auth::generate_chars(64);
    let password = auth::generate_chars(128);
    let hash = auth::hash(&password, &salt);
    assert_eq!(hash, auth::string_to_hash(auth::hash_to_string(hash)));
}

#[test]
fn test_verify_positive() {
    let salt = auth::generate_chars(64);
    let password = auth::generate_chars(128);
    let hash = auth::hash(&password, &salt);
    assert!(auth::verify(&password, &salt, hash));
}

#[test]
fn test_verify_negative() {
    let salt = auth::generate_chars(64);
    let password_a = auth::generate_chars(128);
    let password_b = auth::generate_chars(128);
    let hash_a = auth::hash(&password_a, &salt);
    let hash_b = auth::hash(&password_b, &salt);
    if password_a != password_b {
        assert!(!auth::verify(&password_a, &salt, hash_b));
        assert!(!auth::verify(&password_b, &salt, hash_a));
    }
}