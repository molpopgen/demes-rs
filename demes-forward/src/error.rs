/// Error type.
#[derive(Debug)]
#[non_exhaustive]
pub enum DemesForwardError {
    /// Stores a [`demes::DemesError`].
    DemesError(demes::DemesError),
    /// Errors related to time.
    /// Will be returned if invalid time
    /// values occur after converting
    /// time to generations.
    TimeError(String),
    /// Errors related to invalid deme sizes
    /// arising during application of size change
    /// functions.
    InvalidDemeSize(f64),
    /// Errors related to invalid internal states.
    /// In general, this error indicates a bug
    /// that should be reported.
    InternalError(String),
}

impl From<demes::DemesError> for DemesForwardError {
    fn from(value: demes::DemesError) -> Self {
        Self::DemesError(value)
    }
}

impl std::fmt::Display for DemesForwardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemesForwardError::DemesError(e) => write!(f, "{e:?}"),
            DemesForwardError::TimeError(msg) => write!(f, "time error: {msg}"),
            DemesForwardError::InvalidDemeSize(value) => write!(f, "invalid deme size: {value}"),
            DemesForwardError::InternalError(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for DemesForwardError {}
