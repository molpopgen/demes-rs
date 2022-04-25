use thiserror::Error;

#[derive(Error, Debug)]
pub enum DemesError {
    #[error("epoch time error: {0:?}")]
    EpochTimeError(f64),
    #[error("deme size error: {0:?}")]
    DemeSizeError(f64),
}
