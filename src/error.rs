use thiserror::Error;

/// Error type for this crate.
///
/// The enum fields correspond to
/// the different parts of a [Graph](crate::Graph)
/// defined by the
/// [specification](https://popsim-consortium.github.io/demes-spec-docs/main/introduction.html).
///
/// # Example
///
/// This input is incorrect because the epoch fails
/// to define `start_size` or `end_size`.
/// Attempting to generate a [Graph](crate::Graph)
/// gives [`DemesError::EpochError`](crate::DemesError::EpochError).
///
/// ```
/// let yaml = "
/// time_units: generations
/// demes:
///  - name: A
///    epochs:
///     - end_time: 100
/// ";
/// assert!(matches!(demes::loads(yaml), Err(demes::DemesError::EpochError(_))));
/// ```
#[derive(Error, Debug)]
pub enum DemesError {
    /// Errors related to demes
    #[error("{0:?}")]
    DemeError(String),
    #[error("{0:?}")]
    /// Errors related to epochs
    EpochError(String),
    #[error("{0:?}")]
    /// Top-level errors.
    GraphError(String),
    #[error("{0:?}")]
    /// Errors related to migrations
    MigrationError(String),
    #[error("{0:?}")]
    /// Errors related to pulses
    PulseError(String),
    #[error("{0:?}")]
    /// Errors coming from `serde_yaml`.
    YamlError(serde_yaml::Error),
}

impl From<serde_yaml::Error> for DemesError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::YamlError(error)
    }
}
