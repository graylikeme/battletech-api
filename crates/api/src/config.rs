use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_port")]
    pub port: u16,
    /// Comma-separated list of allowed origins
    #[serde(default)]
    pub allowed_origins: String,
    #[serde(default = "default_schema_version")]
    pub expected_schema_version: i32,
    #[serde(default)]
    pub public_base_url: Option<String>,
}

fn default_port() -> u16 {
    8080
}

fn default_schema_version() -> i32 {
    1
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Load .env file if present (ignore error if not found)
        let _ = dotenvy::dotenv();

        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }

    pub fn allowed_origins_list(&self) -> Vec<String> {
        if self.allowed_origins.is_empty() {
            vec![]
        } else {
            self.allowed_origins
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }
}
