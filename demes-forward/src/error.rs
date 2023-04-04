use thiserror::Error;

/// Error type.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DemesForwardError {
    /// Stores a [`demes::DemesError`].
    #[error("{0:?}")]
    DemesError(demes::DemesError),
    /// Errors related to time.
    /// Will be returned if invalid time
    /// values occur after converting
    /// time to generations.
    #[error("{0:?}")]
    TimeError(String),
    /// Errors related to invalid deme sizes
    /// arising during application of size change
    /// functions.
    #[error("{0:?}")]
    InvalidDemeSize(f64),
    /// Errors related to invalid internal states.
    /// In general, this error indicates a bug
    /// that should be reported.
    #[error("{0:?}")]
    InternalError(String),
}

impl From<demes::DemesError> for DemesForwardError {
    fn from(error: demes::DemesError) -> Self {
        Self::DemesError(error)
    }
}
