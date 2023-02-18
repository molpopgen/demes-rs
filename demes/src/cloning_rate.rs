use crate::error::DemesError;
use crate::traits::Validate;
use serde::{Deserialize, Serialize};

/// The cloning rate of an [`Epoch`](crate::Epoch).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct CloningRate(f64);

impl CloningRate {
    fn validate<F>(&self, f: F) -> Result<(), DemesError>
    where
        F: std::ops::FnOnce(String) -> DemesError,
    {
        if !self.0.is_finite() || self.0.is_sign_negative() || self.0 > 1.0 {
            let msg = format!("cloning rate must be 0.0 <= C <= 1.0, got: {}", self.0);
            Err(f(msg))
        } else {
            Ok(())
        }
    }
}

impl_newtype_traits!(CloningRate);

impl Default for CloningRate {
    fn default() -> Self {
        Self::from(0.0)
    }
}

impl Validate for CloningRate {
    fn validate<F: FnOnce(String) -> DemesError>(&self, err: F) -> Result<(), DemesError> {
        self.validate(err)
    }
}
