use crate::error::DemesError;
use crate::traits::Validate;
use serde::{Deserialize, Serialize};

/// An ancestry proportion.
///
/// This is a newtype wrapper for [`f64`](std::primitive::f64).
///
/// # Interpretation
///
/// With respect to a deme in an *offspring* time step,
/// a proportion is the fraction of ancsestry from a given
/// parental deme.
///
/// # Examples
///
/// ## In `YAML` input
///
/// ### Ancestral proportions of demes
///
/// ```
/// let yaml = "
/// time_units: generations
/// description:
///   An admixed deme appears 100 generations ago.
///   Its initial ancestry is 90% from ancestor1
///   and 10% from ancestor2.
/// demes:
///  - name: ancestor1
///    epochs:
///     - start_size: 50
///       end_time: 100
///  - name: ancestor2
///    epochs:
///     - start_size: 50
///       end_time: 100
///  - name: admixed
///    ancestors: [ancestor1, ancestor2]
///    proportions: [0.9, 0.1]
///    start_time: 100
///    epochs:
///     - start_size: 200
/// ";
/// demes::loads(yaml).unwrap();
/// ```
///
/// ### Pulse proportions
///
/// ```
/// let yaml = "
/// time_units: generations
/// description:
///    Two demes coexist without migration.
///    Sixty three (63) generations ago,
///    deme1 contributes 50% of ancestry
///    to all individuals born in deme2.
/// demes:
///  - name: deme1
///    epochs:
///     - start_size: 50
///  - name: deme2
///    epochs:
///     - start_size: 50
/// pulses:
///  - sources: [deme1]
///    dest: deme2
///    proportions: [0.5]
///    time: 63
/// ";
/// demes::loads(yaml).unwrap();
/// ```
///
/// ## Using rust code
///
/// Normally, one only needs to create a `Proportion` when
/// working with [`GraphBuilder`](crate::GraphBuilder).
///
/// ```
/// let t = demes::Proportion::from(0.5);
/// assert_eq!(t, 0.5);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct Proportion(f64);

impl Proportion {
    fn validate<F>(&self, f: F) -> Result<(), DemesError>
    where
        F: std::ops::FnOnce(String) -> DemesError,
    {
        if !self.0.is_finite() || self.0 <= 0.0 || self.0 > 1.0 {
            let msg = format!("proportions must be 0.0 < p <= 1.0, got: {}", self.0);
            Err(f(msg))
        } else {
            Ok(())
        }
    }
}

impl_newtype_traits!(Proportion);

impl Validate for Proportion {
    fn validate<F: FnOnce(String) -> DemesError>(&self, err: F) -> Result<(), DemesError> {
        self.validate(err)
    }
}
