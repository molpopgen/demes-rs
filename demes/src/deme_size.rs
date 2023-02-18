use crate::error::DemesError;
use crate::traits::Validate;
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
/// Normally, one only needs to create a `DemeSize` when
/// working with [`GraphBuilder`](crate::GraphBuilder).
///
/// ```
/// let t = demes::DemeSize::from(50.0);
/// assert_eq!(t, 50.0);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(from = "f64")]
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

impl_newtype_traits!(DemeSize);

impl Validate for DemeSize {
    fn validate<F: FnOnce(String) -> DemesError>(&self, err: F) -> Result<(), DemesError> {
        self.validate(err)
    }
}
