use crate::error::DemesError;
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
/// ```
/// let t = demes::Proportion::try_from(0.5).unwrap();
/// assert_eq!(t, 0.5);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
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

impl TryFrom<f64> for Proportion {
    type Error = DemesError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let rv = Self(value);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

impl_newtype_traits!(Proportion);

/// Input value for [`Proportion`], used when loading or building graphs.
///
/// # Examples
///
/// ```
/// let t = demes::InputProportion::from(1.0);
/// assert_eq!(t, 1.0);
/// let t = t - 1.0;
/// assert_eq!(t, 0.0);
/// let t = 1.0 + t;
/// assert_eq!(t, 1.0);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct InputProportion(f64);

impl_input_newtype_traits!(InputProportion);

impl TryFrom<InputProportion> for Proportion {
    type Error = DemesError;

    fn try_from(value: InputProportion) -> Result<Self, Self::Error> {
        let rv = Self(value.0);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}
