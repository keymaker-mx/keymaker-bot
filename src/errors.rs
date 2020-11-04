use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseErrors {
    #[error("parsed string is not a command")]
    NotACommand,
    #[error("user is not the admin of the server")]
    NotAllowed,
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    EnvError(#[from] std::env::VarError),
    #[error("unknown parsing error")]
    Unknown,
}