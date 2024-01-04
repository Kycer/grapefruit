use thiserror::Error;

pub type GrapefruitResult<T> = Result<T, GrapefruitError>;

#[derive(Debug, Error)]
pub enum GrapefruitError {
    #[error("PrimaryKey is None!")]
    PrimaryKeyNone(String),

    #[error("MetaObjectNotFound: {0}")]
    MetaObjectNotFound(String),

    #[error("EmptyEntity is None!")]
    EmptyEntity,

    #[error("URL parsing error: `{0}`")]
    UrlParseError(#[from] url::ParseError),

    #[error("Unknown error")]
    Unknown,

    #[error("NoSuchValueError: `{0}`")]
    NoSuchValueError(String),

    #[error("ConvertError: `{0}`")]
    ConvertError(String, String),

    #[error("SerdeError: `{0}`")]
    SerdeError(#[from] serde_json::Error),

    #[error("ObjectValidError: `{0}`")]
    ObjectValidError(String),

    #[error("SqlError: `{0}`")]
    SqlError(#[from] sqlx::Error),

    #[error("PlatformError: `{0}`")]
    PlatformError(String),

    #[error("ValueTypeError: Value type mismatch")]
    ValueTypeError(),
}
