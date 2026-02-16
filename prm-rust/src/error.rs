use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrmError {
    #[error("{field} cannot be blank")]
    BlankField { field: String },

    #[error("{field} must be positive")]
    NonPositive { field: String },

    #[error("{field} cannot be empty")]
    EmptySet { field: String },

    #[error("{entity_type} not found: {id}")]
    NotFound { entity_type: String, id: String },

    #[error("{entity_type} already exists: {identifier}")]
    AlreadyExists {
        entity_type: String,
        identifier: String,
    },

    #[error("Cannot archive self")]
    CannotArchiveSelf,

    #[error("Use log_in_person for in-person interactions")]
    UseInPersonMethod,

    #[error("Use log_remote for remote interactions")]
    UseRemoteMethod,

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type PrmResult<T> = Result<T, PrmError>;
