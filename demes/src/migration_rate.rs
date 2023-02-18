use crate::error::DemesError;
use crate::traits::Validate;
use serde::{Deserialize, Serialize};

/// A migration rate.
///
/// # Examples
///
/// ## Using [`GraphBuilder`](crate::GraphBuilder)
///
/// * [`GraphBuilder::add_symmetric_migration`](crate::GraphBuilder::add_symmetric_migration)
/// * [`GraphBuilder::add_asymmetric_migration`](crate::GraphBuilder::add_asymmetric_migration)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct MigrationRate(f64);

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

impl Default for MigrationRate {
    fn default() -> Self {
        Self::from(0.0)
    }
}

impl Validate for MigrationRate {
    fn validate<F: FnOnce(String) -> DemesError>(&self, err: F) -> Result<(), DemesError> {
        self.validate(err)
    }
}
