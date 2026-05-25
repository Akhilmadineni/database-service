use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid environment variable {key}: {message}")]
    InvalidEnv { key: &'static str, message: String },
}

#[derive(Debug, Error)]
pub enum CapsuleValidationError {
    #[error("invalid {field}: {message}")]
    InvalidField {
        field: &'static str,
        message: String,
    },
    #[error("capsule spec must include at least one role")]
    MissingRoles,
    #[error("capsule spec must include at least one managed schema")]
    MissingManagedSchemas,
    #[error("database owner role `{owner_role}` must be present in roles")]
    OwnerRoleMissing { owner_role: String },
}
