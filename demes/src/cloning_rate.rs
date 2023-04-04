use crate::error::DemesError;
use serde::{Deserialize, Serialize};

/// The cloning rate of an [`Epoch`](crate::Epoch).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
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

impl TryFrom<f64> for CloningRate {
    type Error = DemesError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let rv = Self(value);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

impl_newtype_traits!(CloningRate);

/// Input value for [`CloningRate`], used when loading or building graphs.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct InputCloningRate(f64);

impl From<f64> for InputCloningRate {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl TryFrom<InputCloningRate> for CloningRate {
    type Error = DemesError;

    fn try_from(value: InputCloningRate) -> Result<Self, Self::Error> {
        let rv = Self(value.0);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

impl Default for InputCloningRate {
    fn default() -> Self {
        Self::from(0.0)
    }
}
