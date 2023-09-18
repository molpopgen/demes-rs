use crate::error::DemesError;
use serde::{Deserialize, Serialize};

/// The selfing rate of an [`Epoch`](crate::Epoch).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
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

impl TryFrom<f64> for SelfingRate {
    type Error = DemesError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let rv = Self(value);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

/// Input value for [`SelfingRate`], used when loading or building graphs.
///
/// # Examples
///
/// ```
/// let t = demes::InputSelfingRate::from(1.0);
/// assert_eq!(t, 1.0);
/// let t = t - 1.0;
/// assert_eq!(t, 0.0);
/// let t = 1.0 + t;
/// assert_eq!(t, 1.0);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct InputSelfingRate(f64);

impl_input_newtype_traits!(InputSelfingRate);

impl TryFrom<InputSelfingRate> for SelfingRate {
    type Error = DemesError;

    fn try_from(value: InputSelfingRate) -> Result<Self, Self::Error> {
        let rv = Self(value.0);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

impl Default for InputSelfingRate {
    fn default() -> Self {
        Self::from(0.0)
    }
}
