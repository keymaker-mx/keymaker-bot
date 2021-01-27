use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    EnvError(#[from] std::env::VarError),
    #[error(transparent)]
    TokioSendError(
        #[from] tokio::sync::mpsc::error::SendError<matrix_sdk::events::AnyMessageEventContent>,
    ),
    #[error("Unable to set database singleton")]
    DatabaseSingletonError,
}
