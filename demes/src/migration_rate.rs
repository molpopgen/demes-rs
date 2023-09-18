use crate::error::DemesError;
use serde::{Deserialize, Serialize};

/// A migration rate.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct MigrationRate(f64);

impl TryFrom<f64> for MigrationRate {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let rv = Self(value);
        rv.validate(DemesError::MigrationError)?;
        Ok(rv)
    }
}

/// Input value for [`MigrationRate`], used when loading or building graphs.
///
/// # Examples
///
/// ## Using [`GraphBuilder`](crate::GraphBuilder)
///
/// * [`GraphBuilder::migration`](crate::GraphBuilder::add_migration)
///
/// ```
/// let t = demes::InputMigrationRate::from(0.1);
/// assert_eq!(t, 0.1);
/// let t = t - 1.0;
/// assert_eq!(t, 0.1 - 1.0);
/// let t = 1.0 + t;
/// assert_eq!(t, 0.1 - 1.0 + 1.0);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct InputMigrationRate(f64);

impl_input_newtype_traits!(InputMigrationRate);

impl TryFrom<InputMigrationRate> for MigrationRate {
    type Error = DemesError;

    fn try_from(value: InputMigrationRate) -> Result<Self, Self::Error> {
        Self::try_from(value.0)
    }
}

impl MigrationRate {
    fn validate<F>(&self, f: F) -> Result<(), DemesError>
    where
        F: std::ops::FnOnce(String) -> DemesError,
    {
        if !self.0.is_finite() || self.0.is_sign_negative() || self.0 > 1.0 {
            let msg = format!("migration rate must be 0.0 <= m <= 1.0, got: {}", self.0);
            Err(f(msg))
        } else {
            Ok(())
        }
    }
}

impl_newtype_traits!(MigrationRate);
