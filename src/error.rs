use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("Incomplete reflection detected")]
    UnknownFormat,
}
