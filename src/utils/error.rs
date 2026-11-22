use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid json field")]
    ParsingError,
    #[error(transparent)]
    Request(#[from] wreq::Error),
    #[error(transparent)]
    DeserializingError(#[from] serde_json::Error),
}
