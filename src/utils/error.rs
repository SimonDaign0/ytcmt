use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid json field")]
    ParsingError,
    #[error("Invalid request")]
    Request(#[from] wreq::Error),
}
