use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum LightError {
    #[error("Out of bounds")]
    OutOfBoundsError,

    // Likely not an error, meant to terminate for loops early.
    #[error("Terminated loop early")]
    TerminateEarly,
}
