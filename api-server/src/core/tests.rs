use super::*;

#[test]
fn test_sanity() {
    assert!(true);
}

#[test]
fn test_hash_strings() {
    let salt = auth::generate_chars(64);
    let password = auth::generate_chars(128);
    let hash = auth::hash(&password, &salt[..]);
    let h2s = auth::hash_to_string(hash);
    let s2h = auth::string_to_hash(h2s);

    assert_eq!(hash, s2h);
}

#[test]
fn test_verify() {
    let salt = auth::generate_chars(64);
    let password = auth::generate_chars(128);
    let hash = auth::hash(&password, &salt);
    let verify = auth::verify(&password, &salt, hash);
    assert!(verify);
}
