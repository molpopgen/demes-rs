use thiserror::Error;

#[derive(Error, Debug)]
pub enum DemesError {
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
}
