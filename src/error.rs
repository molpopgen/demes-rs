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
}
