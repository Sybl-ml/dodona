use config::{Config, ConfigFile, Environment};

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
pbkdf2_iterations = "1000"
"#;

#[test]
fn resolve_production_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Production);

    let contents = vec![
        ("app_name".into(), "Sybl".into()),
        ("pepper".into(), "prod".into()),
    ]
    .into_iter()
    .collect();

    let expected = Config(contents);
    assert_eq!(config, expected);
}

#[test]
fn resolve_development_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Development);

    let contents = vec![
        ("app_name".into(), "Dodona".into()),
        ("pepper".into(), "develop".into()),
    ]
    .into_iter()
    .collect();

    let expected = Config(contents);
    assert_eq!(config, expected);
}

#[test]
fn resolve_testing_config() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Testing);

    let contents = vec![
        ("app_name".into(), "Dodona".into()),
        ("conn_str".into(), "localhost".into()),
        ("pepper".into(), "default".into()),
        ("pbkdf2_iterations".into(), "1000".into()),
    ]
    .into_iter()
    .collect();

    let expected = Config(contents);
    assert_eq!(config, expected);
}

#[test]
fn environment_variables_are_updated() {
    let config = ConfigFile::from_str(TEST_CONFIG).resolve(Environment::Testing);
    config.populate_environment();

    let app_name = std::env::var("APP_NAME").ok();
    let conn_str = std::env::var("CONN_STR").ok();
    let pepper = std::env::var("PEPPER").ok();
    let pbkdf2_iterations = std::env::var("PBKDF2_ITERATIONS").ok();

    let contents = vec![
        ("app_name".into(), app_name.unwrap()),
        ("conn_str".into(), conn_str.unwrap()),
        ("pepper".into(), pepper.unwrap()),
        ("pbkdf2_iterations".into(), pbkdf2_iterations.unwrap()),
    ]
    .into_iter()
    .collect();

    let environment = Config(contents);
    assert_eq!(environment, config);
}
