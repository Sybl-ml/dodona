use dodona::config::{ConfigFile, Environment};

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
"#;

#[test]
fn resolve_production_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Production);

    assert_eq!(config.app_name, Some("Sybl".into()));
    assert_eq!(config.conn_str, None);
    assert_eq!(config.pepper, Some("prod".into()));
}

#[test]
fn resolve_development_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Development);

    assert_eq!(config.app_name, Some("Dodona".into()));
    assert_eq!(config.conn_str, None);
    assert_eq!(config.pepper, Some("develop".into()));
}

#[test]
fn resolve_testing_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Testing);

    assert_eq!(config.app_name, Some("Dodona".into()));
    assert_eq!(config.conn_str, Some("localhost".into()));
    assert_eq!(config.pepper, Some("default".into()));
}

#[test]
fn environment_variables_are_updated() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Testing);
    config.populate_environment();

    assert_eq!(std::env::var("APP_NAME"), Ok("Dodona".into()));
    assert_eq!(std::env::var("CONN_STR"), Ok("localhost".into()));
    assert_eq!(std::env::var("PEPPER"), Ok("default".into()));
}
