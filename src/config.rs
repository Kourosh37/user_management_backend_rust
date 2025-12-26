use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub access_token_minutes: i64,
    pub refresh_token_days: i64,
    #[serde(default, deserialize_with = "deserialize_origins")]
    pub cors_allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let settings = Config::builder()
            .set_default("app_host", "0.0.0.0")?
            .set_default("app_port", 8080)?
            .set_default("access_token_minutes", 15)?
            .set_default("refresh_token_days", 7)?
            .set_default("cors_allowed_origins", vec!["http://localhost:3000"])?
            .add_source(Environment::default().separator("__"))
            .build()?;

        settings.try_deserialize()
    }
}

fn deserialize_origins<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Origins {
        One(String),
        Many(Vec<String>),
    }

    match Origins::deserialize(deserializer)? {
        Origins::One(value) => Ok(value
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()),
        Origins::Many(values) => Ok(values),
    }
}
