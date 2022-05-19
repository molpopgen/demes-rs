use thiserror::Error;

#[derive(Error, Debug)]
pub enum DemesError {
    #[error("times must be >= 0.0, got: {0:?}")]
    TimeError(String),
    #[error("{0:?}")]
    DemeError(String),
    #[error("{0:?}")]
    EpochError(String),
    #[error("{0:?}")]
    TopLevelError(String),
    #[error("generation time must be > 0.0, got: {0:?}")]
    GenerationTimeError(f64),
    #[error("{0:?}")]
    MigrationError(String),
    #[error("{0:?}")]
    PulseError(String),
    #[error("{0:?}")]
    YamlError(serde_yaml::Error),
    #[error("{0:?}")]
    UnwoundPanic(String),
}

impl From<serde_yaml::Error> for DemesError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::YamlError(error)
    }
}
