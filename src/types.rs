use serde::{Deserialize, Serialize};

use std::convert::TryFrom;

use crate::DemesError;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
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

impl Default for StartTime {
    fn default() -> Self {
        Self(f64::INFINITY)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "f64")]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct Proportion(f64);

impl TryFrom<f64> for Proportion {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value <= 0.0 || value > 1.0 {
            Err(DemesError::DemeSizeError(value))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TimeInterval {
    start_time: StartTime,
    end_time: EndTime,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SizeFunction {
    #[serde(rename = "constant")]
    CONSTANT,
    #[serde(rename = "exponential")]
    EXPONENTIAL,
}

impl Default for SizeFunction {
    fn default() -> Self {
        Self::EXPONENTIAL
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct CloningRate(f64);

impl TryFrom<f64> for CloningRate {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
            Err(DemesError::CloningRateError(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl Default for CloningRate {
    fn default() -> Self {
        Self::try_from(0.0).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct SelfingRate(f64);

impl TryFrom<f64> for SelfingRate {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
            Err(DemesError::SelfingRateError(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl Default for SelfingRate {
    fn default() -> Self {
        Self::try_from(0.0).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Epoch {
    end_time: EndTime,
    start_size: DemeSize,
    end_size: DemeSize,
    #[serde(default = "SizeFunction::default")]
    size_function: SizeFunction,
    #[serde(default = "CloningRate::default")]
    cloning_rate: CloningRate,
    #[serde(default = "SelfingRate::default")]
    selfing_rate: SelfingRate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deme {
    name: String,
    #[serde(default = "String::default")]
    description: String,
    #[serde(default = "Vec::<String>::default")]
    ancestors: Vec<String>,
    #[serde(default = "Vec::<Proportion>::default")]
    proportions: Vec<Proportion>,
    #[serde(default = "StartTime::default")]
    start_time: StartTime,
    #[serde(default = "Vec::<Epoch>::default")]
    epochs: Vec<Epoch>,
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
    #[should_panic]
    fn test_deme_size_zero() {
        let yaml = "---\n0.0\n".to_string();
        let _: DemeSize = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    fn test_size_function() {
        let yaml = "---\nexponential\n".to_string();
        let sf: SizeFunction = serde_yaml::from_str(&yaml).unwrap();
        match sf {
            SizeFunction::EXPONENTIAL => (),
            SizeFunction::CONSTANT => panic!("expected SizeFunction::Exponential"),
        }
        let yaml = "---\nconstant\n".to_string();
        let sf: SizeFunction = serde_yaml::from_str(&yaml).unwrap();
        match sf {
            SizeFunction::EXPONENTIAL => panic!("expected SizeFunction::Constant"),
            SizeFunction::CONSTANT => (),
        }
    }

    #[test]
    fn test_valid_cloning_rate() {
        let yaml = "---\n0.0\n".to_string();
        let cr: CloningRate = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(cr.0, 0.0);
        let yaml = "---\n1.0\n".to_string();
        let cr: CloningRate = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(cr.0, 1.0);
    }

    #[test]
    #[should_panic]
    fn test_negative_cloning_rate() {
        let yaml = "---\n-0.0\n".to_string();
        let _: CloningRate = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_selfing_rates_above_one() {
        let yaml = "---\n1.1\n".to_string();
        let _: CloningRate = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    fn test_valid_selfing_rate() {
        let yaml = "---\n0.0\n".to_string();
        let cr: SelfingRate = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(cr.0, 0.0);
        let yaml = "---\n1.0\n".to_string();
        let cr: SelfingRate = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(cr.0, 1.0);
    }

    #[test]
    #[should_panic]
    fn test_negative_selfing_rate() {
        let yaml = "---\n-0.0\n".to_string();
        let _: SelfingRate = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_cloning_rates_above_one() {
        let yaml = "---\n1.1\n".to_string();
        let _: SelfingRate = serde_yaml::from_str(&yaml).unwrap();
    }
}
