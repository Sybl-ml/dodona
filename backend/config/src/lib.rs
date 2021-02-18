//! Deals with loading configuration files for Dodona.
//!
//! This allows for specific environment level configuration instead of a simple `.env` file,
//! meaning different variables can be used depending on the server state.
//!
//! For example, a local instance of MongoDB can be used for running tests, whereas the Atlas
//! instance can be used for production purposes.

use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::path::Path;
use std::str::FromStr;

#[macro_use]
extern crate serde;

/// Defines which subconfig to use for overwriting.
#[derive(Copy, Clone, Debug)]
pub enum Environment {
    /// Uses the `[production]` section
    Production,
    /// Uses the `[development]` section
    Development,
    /// Uses the `[testing]` section
    Testing,
}

/// Defines the structure of a single section of a configuration file.
#[derive(Eq, PartialEq, Debug, Default, Deserialize)]
pub struct Config(pub HashMap<String, String>);

impl Config {
    /// Updates a config's values if the other config has them set.
    ///
    /// This is used to determine precedence of the config values, allowing global ones to be
    /// overwritten by a more specific one.
    ///
    /// # Examples
    ///
    /// ```
    /// use config::Config;
    ///
    /// let config_contents = vec![
    ///     ("app_name".into(), "app_name".into()),
    ///     ("conn_str".into(), "localhost".into())
    /// ].into_iter().collect();
    ///
    /// let mut config = Config(config_contents);
    ///
    /// let specific_contents = vec![
    ///     ("conn_str".into(), "mongo_uri".into()),
    ///     ("pepper".into(), "pepper".into())
    /// ].into_iter().collect();
    ///
    /// let specific = Config(specific_contents);
    ///
    /// config.or(specific);
    ///
    /// let expected_contents = vec![
    ///     ("app_name".into(), "app_name".into()),
    ///     ("conn_str".into(), "mongo_uri".into()),
    ///     ("pepper".into(), "pepper".into())
    /// ].into_iter().collect();
    ///
    /// let expected = Config(expected_contents);
    ///
    /// assert_eq!(config, expected);
    /// ```
    pub fn or(&mut self, config: Config) {
        for (key, value) in config.0 {
            self.0.insert(key, value);
        }
    }

    /// Populates the environment variables based on the values in the configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use config::Config;
    ///
    /// let expected_contents = vec![
    ///     ("app_name".into(), "app_name".into()),
    ///     ("conn_str".into(), "localhost".into()),
    /// ].into_iter().collect();
    ///
    /// let config = Config(expected_contents);
    ///
    /// config.populate_environment();
    ///
    /// assert_eq!(std::env::var("APP_NAME"), Ok("app_name".into()));
    /// assert_eq!(std::env::var("CONN_STR"), Ok("localhost".into()));
    ///
    /// assert!(std::env::var("PEPPER").is_err());
    /// assert!(std::env::var("PBKDF2_ITERATIONS").is_err());
    /// ```
    pub fn populate_environment(&self) {
        for (key, value) in &self.0 {
            std::env::set_var(key.to_uppercase(), value);
        }
    }
}

/// Defines the full structure of the configuration file, along with the sections.
#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    /// The `[global]` configuration section
    pub global: Option<Config>,
    /// The `[development]` configuration section
    pub development: Option<Config>,
    /// The `[production]` configuration section
    pub production: Option<Config>,
    /// The `[testing]` configuration section
    pub testing: Option<Config>,
}

impl ConfigFile {
    /// Searches the file system for a configuration file.
    ///
    /// Initially begins the search from the current directory, before moving upwards until the
    /// root directory. Panics if it cannot find a `config.toml` before it hits `/`.
    pub fn from_filesystem() -> Self {
        // Get the current directory
        let mut current = env::current_dir().unwrap();

        loop {
            // Turn /foo/bar into /foo/bar/config.toml
            let config_path = current.join("config.toml");

            // If the file exists, parse a config from it
            if Path::new(&config_path).exists() {
                return Self::from_file(&config_path);
            }

            // If we didn't find anything even at /config.toml, panic
            if !current.pop() {
                panic!("Failed to find a `config.toml` file.");
            }
        }
    }

    /// Reads a `ConfigFile` from a given filename.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use config::ConfigFile;
    ///
    /// let config_file = ConfigFile::from_file("config.toml");
    /// ```
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Self {
        let contents = std::fs::read_to_string(filename).unwrap();
        Self::from_str(&contents).unwrap()
    }

    /// Resolves a `ConfigFile` into a single `Config` given the environment that is running.
    ///
    /// This allows any subconfigurations to override the global one, allowing for different Mongo
    /// URI strings for testing for example.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use config::{Config, ConfigFile, Environment};
    ///
    /// let config = r#"
    /// [global]
    /// app_name = "Dodona"
    /// pepper = "pepper"
    ///
    /// [production]
    /// pbkdf2_iterations = "10000"
    ///
    /// [development]
    /// pepper = "dev_pepper"
    ///
    /// [testing]
    /// conn_str = "localhost"
    ///
    /// "#;
    ///
    /// let config_file = ConfigFile::from_str(config).unwrap();
    /// let development = config_file.resolve(Environment::Development);
    ///
    /// let expected_contents = vec![
    ///     ("app_name".into(), "Dodona".into()),
    ///     ("pepper".into(), "dev_pepper".into())
    /// ].into_iter().collect();
    ///
    /// let expected = Config(expected_contents);
    ///
    /// assert_eq!(development, expected);
    /// ```
    pub fn resolve(self, environment: Environment) -> Config {
        log::info!("Resolving config: {:?}", environment);

        // Start with defaults
        let mut config = Config::default();

        // Check if we have a global config
        if let Some(global) = self.global {
            config.or(global);
        }

        // Get the subconfig for the environment
        let subconfig = match environment {
            Environment::Production => self.production,
            Environment::Development => self.development,
            Environment::Testing => self.testing,
        };

        // Override the global and defaults if we have a subconfig
        if let Some(subconfig) = subconfig {
            config.or(subconfig);
        }

        log::info!("Config values: {:#?}", config);

        config
    }
}

impl FromStr for ConfigFile {
    type Err = Infallible;

    /// Parses a `ConfigFile` from a given string.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::str::FromStr;
    /// use config::ConfigFile;
    ///
    /// let config = r#"
    /// [global]
    /// app_name = "Dodona"
    /// pepper = "pepper"
    ///
    /// [development]
    /// pepper = "dev_pepper"
    ///
    /// [testing]
    /// conn_str = "localhost"
    /// "#;
    ///
    /// let config_file = ConfigFile::from_str(config).unwrap();
    ///
    /// assert!(config_file.global.is_some());
    /// assert!(config_file.development.is_some());
    /// assert!(config_file.testing.is_some());
    ///
    /// assert!(config_file.production.is_none());
    /// ```
    fn from_str(contents: &str) -> Result<Self, Self::Err> {
        Ok(toml::from_str(&contents).unwrap())
    }
}

/// Loads the configuration for a given environment into environment variables.
///
/// Given the current environment, loads the configuration file and resolves it based on the given
/// environment, before populating the environment variables with the values contained.
pub fn load(environment: Environment) {
    let config = ConfigFile::from_filesystem();
    let resolved = config.resolve(environment);
    resolved.populate_environment();
}
