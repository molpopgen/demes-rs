use serde::{Deserialize, Serialize};

use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Serialize)]
#[repr(transparent)]
/// Representation of "time" values.
/// The underlying `f64` must be non-negative, non-NaN.
///
/// # Examples
///
/// The type is constructed from an `f64`:
///
/// ```
/// let t = demes::EpochTime::try_from(1.0).unwrap();
/// ```
///
/// ```{should_panic}
/// let t = demes::EpochTime::try_from(-1.0).unwrap();
/// ```
///
/// ```{should_panic}
/// let t = demes::EpochTime::try_from(f64::NAN).unwrap();
/// ```
pub struct EpochTime(f64);

#[derive(Clone, Copy, Debug, Serialize)]
#[repr(transparent)]
/// Representation of deme sizes.
/// The underlying `f64` must be non-negative, non-NaN.
///
/// # Examples
///
/// The type is constructed from an `f64`:
///
/// ```
/// let t = demes::DemeSize::try_from(1.0).unwrap();
/// ```
///
/// ```{should_panic}
/// let t = demes::DemeSize::try_from(-1.0).unwrap();
/// ```
///
/// ```{should_panic}
/// let t = demes::DemeSize::try_from(f64::NAN).unwrap();
/// ```
pub struct DemeSize(f64);

impl_f64_newtypes!(EpochTime, EpochTimeError);
impl_f64_newtypes!(DemeSize, DemeSizeError);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TimeInterval {
    start_time: EpochTime,
    end_time: EpochTime,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Epoch {
    #[serde(flatten)]
    time_interval: TimeInterval,
}

impl Epoch {
    pub fn start_time(&self) -> EpochTime {
        self.time_interval.start_time
    }

    pub fn end_time(&self) -> EpochTime {
        self.time_interval.end_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_negative_epoch_start_time() {
        let yaml = "---\nstart_time: -1.0\nend_time: 1.1\n".to_string();
        let ti: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ti.start_time.0, 1.0);
        assert_eq!(ti.end_time.0, 1.1);
    }

    #[test]
    #[should_panic]
    fn test_nan_epoch_start_time() {
        let yaml = "---\nstart_time: .nan\nend_time: 1.1\n".to_string();
        let ti: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ti.start_time.0, 1.0);
        assert_eq!(ti.end_time.0, 1.1);
    }

    #[test]
    #[should_panic]
    fn test_negative_epoch_end() {
        let yaml = "---\nstart_time: 1.0\nend_time: -1.1\n".to_string();
        let ti: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ti.start_time.0, 1.0);
        assert_eq!(ti.end_time.0, 1.1);
    }

    #[test]
    #[should_panic]
    fn test_nan_epoch_end() {
        let yaml = "---\nstart_time: 1.0\nend_time: .nan\n".to_string();
        let ti: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ti.start_time.0, 1.0);
        assert_eq!(ti.end_time.0, 1.1);
    }

    #[test]
    fn test_time_interval() {
        let yaml = "---\nstart_time: 1.0\nend_time: 1.1\n".to_string();
        let ti: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ti.start_time.0, 1.0);
        assert_eq!(ti.end_time.0, 1.1);
    }

    #[test]
    fn test_epoch() {
        let yaml = "---\nstart_time: 1.0\nend_time: 1.1\n".to_string();
        let epoch: Epoch = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(epoch.time_interval.start_time.0, 1.0);
        assert_eq!(epoch.time_interval.end_time.0, 1.1);
    }
}
