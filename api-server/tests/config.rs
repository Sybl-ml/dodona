use std::str::FromStr;

use dodona::config::{Config, ConfigFile, Environment};

static TEST_CONFIG: &str = r#"
[global]
app_name = "Dodona"
pepper = "default"

[production]
app_name = "Sybl"
pepper = "prod"

[development]
pepper = "develop"

[testing]
conn_str = "localhost"
pbkdf2_iterations = 1000
"#;

#[test]
fn resolve_production_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Production);

    let expected = Config {
        app_name: Some("Sybl".into()),
        conn_str: None,
        pepper: Some("prod".into()),
        pbkdf2_iterations: None,
    };

    assert_eq!(config, expected);
}

#[test]
fn resolve_development_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Development);

    let expected = Config {
        app_name: Some("Dodona".into()),
        conn_str: None,
        pepper: Some("develop".into()),
        pbkdf2_iterations: None,
    };

    assert_eq!(config, expected);
}

#[test]
fn resolve_testing_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Testing);

    let expected = Config {
        app_name: Some("Dodona".into()),
        conn_str: Some("localhost".into()),
        pepper: Some("default".into()),
        pbkdf2_iterations: Some(1000),
    };

    assert_eq!(config, expected);
}

#[test]
fn environment_variables_are_updated() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Testing);
    config.populate_environment();

    let app_name = std::env::var("APP_NAME").ok();
    let conn_str = std::env::var("CONN_STR").ok();
    let pepper = std::env::var("PEPPER").ok();
    let pbkdf2_iterations = std::env::var("PBKDF2_ITERATIONS")
        .ok()
        .map(|x| u32::from_str(&x).unwrap());

    let environment = Config {
        app_name,
        conn_str,
        pepper,
        pbkdf2_iterations,
    };

    assert_eq!(environment, config);
}
