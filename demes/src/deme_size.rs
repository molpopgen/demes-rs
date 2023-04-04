use crate::error::DemesError;
use serde::{Deserialize, Serialize};

/// The size of a [`Deme`](crate::Deme) at a given [`Time`](crate::Time).
///
/// This is a newtype wrapper for [`f64`](std::primitive::f64).
///
/// # Notes
///
/// * The size may take on non-integer values.
///
/// # Examples
///
/// ## In a `YAML` record
///
/// ```
/// let yaml = "
/// time_units: years
/// generation_time: 25
/// description:
///   A deme of 50 individuals that grew to 100 individuals
///   in the last 100 years.
/// demes:
///  - name: deme
///    epochs:
///     - start_size: 50
///       end_time: 100
///     - start_size: 50
///       end_size: 100
/// ";
/// demes::loads(yaml).unwrap();
/// ```
///
/// ## Using rust code
///
/// ```
/// let t = demes::DemeSize::try_from(50.0).unwrap();
/// assert_eq!(t, 50.0);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "f64")]
#[repr(transparent)]
pub struct DemeSize(f64);

impl DemeSize {
    fn validate<F>(&self, f: F) -> Result<(), DemesError>
    where
        F: std::ops::FnOnce(String) -> DemesError,
    {
        if self.0.is_nan() || self.0.is_infinite() || self.0 <= 0.0 {
            let msg = format!("deme sizes must be 0 <= d < Infinity, got: {}", self.0);
            Err(f(msg))
        } else {
            Ok(())
        }
    }
}

impl TryFrom<f64> for DemeSize {
    type Error = DemesError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let rv = Self(value);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

impl_newtype_traits!(DemeSize);

/// Input value for [`DemeSize`], used when loading or building graphs.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct InputDemeSize(f64);

impl From<f64> for InputDemeSize {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl TryFrom<InputDemeSize> for DemeSize {
    type Error = DemesError;

    fn try_from(value: InputDemeSize) -> Result<Self, Self::Error> {
        let rv = Self(value.0);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

impl From<InputDemeSize> for f64 {
    fn from(value: InputDemeSize) -> Self {
        value.0
    }
}
