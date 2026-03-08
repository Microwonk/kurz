use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub shortener: ShortenerConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub password: String,
    pub session_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShortenerConfig {
    pub random_string_length: usize,
    pub random_word_count: usize,
    pub word_separator: String,
}

impl Default for ShortenerConfig {
    fn default() -> Self {
        Self {
            random_string_length: 6,
            random_word_count: 3,
            word_separator: "-".into(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct UiConfig {
    pub accent_color: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        toml::from_str(&fs::read_to_string(path)?).map_err(Into::into)
    }
}
