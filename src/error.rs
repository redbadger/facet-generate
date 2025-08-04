use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("Incomplete reflection detected")]
    UnknownFormat,
}

impl Error {
    /// Provides a longer description of the possible cause of an error during tracing.
    #[must_use]
    pub fn explanation(&self) -> String {
        match self {
            Error::UnknownFormat => {
                "An internal error that indicates an incomplete reflection of the annotated types."
                    .to_string()
            }
        }
    }
}
