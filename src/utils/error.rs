use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid json section")]
    ParsingError,
    #[error(transparent)]
    Request(#[from] wreq::Error),
    #[error(transparent)]
    DeserializingError(#[from] serde_json::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
