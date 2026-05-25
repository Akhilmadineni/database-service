use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid environment variable {key}: {message}")]
    InvalidEnv { key: &'static str, message: String },
}
