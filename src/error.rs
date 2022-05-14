use thiserror::Error;

#[derive(Error, Debug)]
pub enum DemesError {
    #[error("times must be >= 0.0, got: {0:?}")]
    TimeError(f64),
    #[error("start_time must be > 0.0, got: {0:?}")]
    StartTimeError(f64),
    #[error("end_time must be <= t < Infinity, got: {0:?}")]
    EndTimeError(f64),
    #[error("deme sizes must be 0 <= d < Infinity, got: {0:?}")]
    DemeSizeError(f64),
    #[error("proportions must be 0.0 < p <= 1.0, got: {0:?}")]
    ProportionError(f64),
    #[error("cloning rate must be 0.0 <= C <= 1.0, got: {0:?}")]
    CloningRateError(f64),
    #[error("selfing rate must be 0.0 <= S <= 1.0, got: {0:?}")]
    SelfingRateError(f64),
    #[error("migration rate must be 0.0 <= m <= 1.0, got: {0:?}")]
    MigrationRateError(f64),
    #[error("{0:?}")]
    DemeError(String),
    #[error("{0:?}")]
    EpochError(String),
    #[error("{0:?}")]
    AncestorError(String),
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
}

impl From<serde_yaml::Error> for DemesError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::YamlError(error)
    }
}
