use thiserror::Error;

#[derive(Error, Debug)]
pub enum DemesError {
    #[error("{0:?}")]
    DemeError(String),
    #[error("{0:?}")]
    EpochError(String),
    #[error("{0:?}")]
    GraphError(String),
    #[error("generation time must be > 0.0, got: {0:?}")]
    GenerationTimeError(f64),
    #[error("{0:?}")]
    MigrationError(String),
    #[error("{0:?}")]
    PulseError(String),
    #[error("{0:?}")]
    YamlError(serde_yaml::Error),
}

impl From<serde_yaml::Error> for DemesError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::YamlError(error)
    }
}
