use crate::error::DemesError;
use crate::traits::Validate;
use serde::{Deserialize, Serialize};

/// The selfing rate of an [`Epoch`](crate::Epoch).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct SelfingRate(f64);

impl SelfingRate {
    fn validate<F>(&self, f: F) -> Result<(), DemesError>
    where
        F: std::ops::FnOnce(String) -> DemesError,
    {
        if !self.0.is_finite() || self.0.is_sign_negative() || self.0 > 1.0 {
            let msg = format!("selfing rate must be 0.0 <= S <= 1.0, got: {}", self.0);
            Err(f(msg))
        } else {
            Ok(())
        }
    }
}

impl_newtype_traits!(SelfingRate);

impl Default for SelfingRate {
    fn default() -> Self {
        Self::from(0.0)
    }
}

impl Validate for SelfingRate {
    fn validate<F: FnOnce(String) -> DemesError>(&self, err: F) -> Result<(), DemesError> {
        self.validate(err)
    }
}
