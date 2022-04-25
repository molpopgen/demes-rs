use serde::{Deserialize, Serialize};

use std::convert::TryFrom;

use crate::DemesError;

#[derive(Clone, Copy, Debug, Serialize)]
#[repr(transparent)]
pub struct StartTime(f64);

impl TryFrom<f64> for StartTime {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_nan() || value <= 0.0 {
            Err(DemesError::StartTimeError(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl_deserialize_for_try_from_f64!(StartTime);

#[derive(Clone, Copy, Debug, Serialize)]
#[repr(transparent)]
pub struct EndTime(f64);

impl TryFrom<f64> for EndTime {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_nan() || value.is_infinite() || value < 0.0 {
            Err(DemesError::EndTimeError(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl_deserialize_for_try_from_f64!(EndTime);

#[derive(Clone, Copy, Debug, Serialize)]
#[repr(transparent)]
/// Representation of deme sizes.
/// The underlying `f64` must be non-negative, non-NaN.
pub struct DemeSize(f64);

impl TryFrom<f64> for DemeSize {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_nan() || value.is_infinite() || value <= 0.0 {
            Err(DemesError::DemeSizeError(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl_deserialize_for_try_from_f64!(DemeSize);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TimeInterval {
    start_time: StartTime,
    end_time: EndTime,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Epoch {
    #[serde(flatten)]
    time_interval: TimeInterval,
}

impl Epoch {
    pub fn start_time(&self) -> StartTime {
        self.time_interval.start_time
    }

    pub fn end_time(&self) -> EndTime {
        self.time_interval.end_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_negative_start_time() {
        let yaml = "---\nstart_time: -1.0\nend_time: 1.1\n".to_string();
        let _: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_nan_start_time() {
        let yaml = "---\nstart_time: .nan\nend_time: 1.1\n".to_string();
        let _: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    fn test_zero_end_time() {
        let yaml = "---\nstart_time: 1.0\nend_time: 0.\n".to_string();
        let ts: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ts.start_time.0, 1.0);
        assert_eq!(ts.end_time.0, 0.0);
    }

    #[test]
    #[should_panic]
    fn test_zero_start_time() {
        let yaml = "---\nstart_time: 0.0\nend_time: 1.1\n".to_string();
        let _: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_negative_end_time() {
        let yaml = "---\nstart_time: 1.0\nend_time: -1.1\n".to_string();
        let _: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_infinite_end_time() {
        let yaml = "---\nstart_time: 1.0\nend_time: .Inf\n".to_string();
        let _: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_nan_epoch_end_time() {
        let yaml = "---\nstart_time: 1.0\nend_time: .nan\n".to_string();
        let _: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
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

    #[test]
    #[should_panic]
    fn test_deme_size_zero() {
        let yaml = "---\n0.0\n".to_string();
        let _: DemeSize = serde_yaml::from_str(&yaml).unwrap();
    }
}
