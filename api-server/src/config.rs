//! Deals with loading configuration files for Dodona.
//!
//! This allows for specific environment level configuration instead of a simple `.env` file,
//! meaning different variables can be used depending on the server state.
//!
//! For example, a local instance of MongoDB can be used for running tests, whereas the Atlas
//! instance can be used for production purposes.

/// Defines which subconfig to use for overwriting.
#[derive(Debug)]
pub enum Environment {
    /// Uses the `[production]` section
    Production,
    /// Uses the `[development]` section
    Development,
    /// Uses the `[testing]` section
    Testing,
}

/// Defines what variables can be configured in the file and their types.
#[derive(Eq, PartialEq, Debug, Default, Deserialize)]
pub struct Config {
    /// The name of the app to use for MongoDB
    pub app_name: Option<String>,
    /// The connection URI for the Mongo database instance
    pub conn_str: Option<String>,
    /// The pepper to use for hashing purposes
    pub pepper: Option<String>,
    /// The number of iterations to use for hashing purposes
    pub pbkdf2_iterations: Option<u32>,
}

impl Config {
    /// Updates a config's values if the other config has them set.
    ///
    /// This is used to determine precedence of the config values, allowing global ones to be
    /// overwritten by a more specific one.
    ///
    /// # Examples
    ///
    /// ```
    /// use dodona::config::Config;
    ///
    /// let mut config = Config {
    ///     app_name: Some("app_name".into()),
    ///     conn_str: Some("localhost".into()),
    ///     pepper: None,
    ///     pbkdf2_iterations: None,
    /// };
    ///
    /// let specific = Config {
    ///     app_name: None,
    ///     conn_str: Some("mongo_uri".into()),
    ///     pepper: Some("pepper".into()),
    ///     pbkdf2_iterations: None,
    /// };
    ///
    /// config.or(specific);
    ///
    /// let expected = Config {
    ///     app_name: Some("app_name".into()),
    ///     conn_str: Some("mongo_uri".into()),
    ///     pepper: Some("pepper".into()),
    ///     pbkdf2_iterations: None,
    /// };
    ///
    /// assert_eq!(config, expected);
    /// ```
    pub fn or(&mut self, config: Config) {
        if config.app_name.is_some() {
            self.app_name = config.app_name;
        }

        if config.conn_str.is_some() {
            self.conn_str = config.conn_str;
        }

        if config.pepper.is_some() {
            self.pepper = config.pepper;
        }

        if config.pbkdf2_iterations.is_some() {
            self.pbkdf2_iterations = config.pbkdf2_iterations;
        }
    }

    /// Populates the environment variables based on the values in the configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use dodona::config::Config;
    ///
    /// let config = Config {
    ///     app_name: Some("app_name".into()),
    ///     conn_str: Some("localhost".into()),
    ///     pepper: None,
    ///     pbkdf2_iterations: None,
    /// };
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
        if let Some(app_name) = &self.app_name {
            std::env::set_var("APP_NAME", app_name);
        }

        if let Some(conn_str) = &self.conn_str {
            std::env::set_var("CONN_STR", conn_str);
        }

        if let Some(pepper) = &self.pepper {
            std::env::set_var("PEPPER", pepper);
        }

        if let Some(pbkdf2_iterations) = &self.pbkdf2_iterations {
            std::env::set_var("PBKDF2_ITERATIONS", pbkdf2_iterations.to_string());
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
    /// Reads a `ConfigFile` from a given filename.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dodona::config::ConfigFile;
    ///
    /// let config_file = ConfigFile::from_file("config.toml");
    /// ```
    pub fn from_file(filename: &str) -> Self {
        let contents = std::fs::read_to_string(filename).unwrap();
        Self::from_str(&contents)
    }

    /// Parses a `ConfigFile` from a given string.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dodona::config::ConfigFile;
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
    /// let config_file = ConfigFile::from_str(config);
    ///
    /// assert!(config_file.global.is_some());
    /// assert!(config_file.development.is_some());
    /// assert!(config_file.testing.is_some());
    ///
    /// assert!(config_file.production.is_none());
    /// ```
    pub fn from_str(contents: &str) -> Self {
        toml::from_str(&contents).unwrap()
    }

    /// Resolves a `ConfigFile` into a single `Config` given the environment that is running.
    ///
    /// This allows any subconfigurations to override the global one, allowing for different Mongo
    /// URI strings for testing for example.
    ///
    /// # Examples
    ///
    /// ```
    /// use dodona::config::{Config, ConfigFile, Environment};
    ///
    /// let config = r#"
    /// [global]
    /// app_name = "Dodona"
    /// pepper = "pepper"
    ///
    /// [production]
    /// pbkdf2_iterations = 10000
    ///
    /// [development]
    /// pepper = "dev_pepper"
    ///
    /// [testing]
    /// conn_str = "localhost"
    ///
    /// "#;
    ///
    /// let config_file = ConfigFile::from_str(config);
    /// let development = config_file.resolve(Environment::Development);
    ///
    /// let expected = Config {
    ///     app_name: Some("Dodona".into()),
    ///     conn_str: None,
    ///     pepper: Some("dev_pepper".into()),
    ///     pbkdf2_iterations: None,
    /// };
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
