use crate::DemesForwardError;

/// The current size of a deme.
///
/// Unlike [`demes::DemeSize`], this type
/// allows for values of 0.0, which means
/// that there are no individuals in the
/// deme at the current time.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct CurrentSize(f64);

impl TryFrom<f64> for CurrentSize {
    type Error = DemesForwardError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_sign_positive() && value.is_finite() {
            Ok(Self(value))
        } else {
            Err(DemesForwardError::InvalidDemeSize(value))
        }
    }
}

impl TryFrom<demes::DemeSize> for CurrentSize {
    type Error = DemesForwardError;

    fn try_from(value: demes::DemeSize) -> Result<Self, Self::Error> {
        Self::try_from(f64::from(value))
    }
}

impl PartialEq<CurrentSize> for f64 {
    fn eq(&self, other: &CurrentSize) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<f64> for CurrentSize {
    fn eq(&self, other: &f64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<CurrentSize> for f64 {
    fn partial_cmp(&self, other: &CurrentSize) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl PartialOrd<f64> for CurrentSize {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
