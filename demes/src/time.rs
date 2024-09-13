use crate::error::DemesError;
use serde::{Deserialize, Serialize};

pub(crate) fn to_generations(time: Time, generation_time: GenerationTime) -> Time {
    let t = f64::from(time);
    let g = f64::from(generation_time);

    (t / g).try_into().unwrap()
}

/// Convert a time value into generations, rounding output to closest integer.
///
/// # Note
///
/// Rounds result using [`f64::round`].
pub fn round_time_to_integer_generations(time: Time, generation_time: GenerationTime) -> Time {
    let t = f64::from(time);
    let g = f64::from(generation_time);

    (t / g).round().try_into().unwrap()
}

/// Store time values.
///
/// This is a newtype wrapper for [`f64`](std::primitive::f64).
///
/// # Notes
///
/// * The units are in the [`TimeUnits`](crate::TimeUnits)
///   of the [`Graph`](crate::Graph).
/// * Invalid values are caught when a `Graph` is
///   resolved.  Funcions that generate resolved graphs are:
///    - [`loads`](crate::loads)
///    - [`load`](crate::load)
///    - [`GraphBuilder::resolve`](crate::GraphBuilder::resolve)
///
/// # Examples
///
/// ## In a `YAML` record
///
/// ```
/// let yaml = "
/// time_units: years
/// generation_time: 25
/// description: A deme that existed until 20 years ago.
/// demes:
///  - name: deme
///    epochs:
///     - start_size: 50
///       end_time: 20
/// ";
/// demes::loads(yaml).unwrap();
/// ```
///
/// ## Using rust code
///
/// The only method to create a `Time` is to
/// apply `TryFrom<f64>`:
///
/// ```
/// let t = demes::Time::try_from(0.0).unwrap();
/// assert_eq!(t, 0.0);
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "TimeTrampoline")]
#[serde(into = "TimeTrampoline")]
pub struct Time(f64);

impl_newtype_traits!(Time);

impl TryFrom<f64> for Time {
    type Error = DemesError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let rv = Self(value);
        rv.validate(DemesError::ValueError)?;

        Ok(rv)
    }
}

/// Input value for [`Time`], used when loading or building graphs.
///
/// # Examples
///
/// ```
/// let t = demes::InputTime::from(1.0);
/// assert_eq!(t, 1.0);
/// let t = t - 1.0;
/// assert_eq!(t, 0.0);
/// let t = 1.0 + t;
/// assert_eq!(t, 1.0);
/// ```
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
#[repr(transparent)]
#[serde(try_from = "TimeTrampoline")]
pub struct InputTime(f64);

impl InputTime {
    pub(crate) fn default_deme_start_time() -> Self {
        Self(f64::INFINITY)
    }
    pub(crate) fn default_epoch_end_time() -> Self {
        Self(0.0)
    }
    pub(crate) fn is_valid_deme_start_time(&self) -> bool {
        self.0 > 0.0
    }
    pub(crate) fn err_if_not_valid_deme_start_time(&self) -> Result<(), DemesError> {
        if self.is_valid_deme_start_time() {
            Ok(())
        } else {
            let msg = format!("start_time must be > 0.0, got: {}", self.0);
            Err(DemesError::DemeError(msg))
        }
    }
}

impl_input_newtype_traits!(InputTime);

impl TryFrom<InputTime> for Time {
    type Error = DemesError;

    fn try_from(value: InputTime) -> Result<Self, Self::Error> {
        let rv = Self(value.0);
        rv.validate(DemesError::ValueError)?;
        Ok(rv)
    }
}

impl From<Time> for InputTime {
    fn from(value: Time) -> Self {
        Self(value.into())
    }
}

/// Generation time.
///
/// If [`TimeUnits`] are in generations, this value
/// must be 1.0.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct GenerationTime(f64);

impl_newtype_traits!(GenerationTime);

/// The time units of a graph
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(from = "String")]
#[serde(into = "String")]
pub enum TimeUnits {
    #[allow(missing_docs)]
    Generations,
    #[allow(missing_docs)]
    Years,
    /// A "custom" time unit.  It is assumed
    /// that client code knows what to do with this.
    Custom(String),
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct HashableTime(Time);

/// A half-open time interval `[present, past)`.
#[derive(Clone, Copy, Debug)]
pub struct TimeInterval {
    start_time: Time,
    end_time: Time,
}

impl std::fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}]", self.start_time, self.end_time)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum TimeTrampoline {
    Infinity(String),
    Float(f64),
}

// Workhorse behing Graph::to_generations
pub(crate) fn convert_resolved_time_to_generations<F>(
    generation_time: GenerationTime,
    rounding: fn(Time, GenerationTime) -> Time,
    f: F,
    message: &str,
    input: Option<Time>,
) -> Result<Time, DemesError>
where
    F: std::ops::FnOnce(String) -> DemesError,
{
    match input {
        Some(value) => {
            if value.0.is_infinite() {
                return Ok(value);
            }
            let time = rounding(value, generation_time);

            if time.0.is_finite() && time >= 0.0 {
                Ok(time)
            } else {
                Err(f("rounding resulted in invalid time".to_string()))
            }
        }
        None => Err(f(message.to_string())),
    }
}

impl Time {
    pub(crate) fn is_valid_pulse_time(&self) -> bool {
        self.0.is_sign_positive() && !self.0.is_infinite()
    }

    fn validate<F>(&self, f: F) -> Result<(), DemesError>
    where
        F: std::ops::FnOnce(String) -> DemesError,
    {
        if self.0.is_nan() || self.0.is_sign_negative() {
            Err(f(format!("invalid time value: {}", self.0)))
        } else {
            Ok(())
        }
    }
}

impl GenerationTime {
    fn validate<F: FnOnce(String) -> DemesError>(&self, err: F) -> Result<(), DemesError> {
        if !self.0.is_finite() || !self.0.is_sign_positive() || !self.gt(&0.0) {
            Err(err(format!("generation time must be > 0.0, got: {self}")))
        } else {
            Ok(())
        }
    }
}

impl TryFrom<f64> for GenerationTime {
    type Error = DemesError;
    fn try_from(value: f64) -> Result<GenerationTime, Self::Error> {
        let rv = Self(value);
        rv.validate(Self::Error::GraphError)?;
        Ok(rv)
    }
}

/// Input value for [`GenerationTime`], used when loading or building graphs.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(from = "f64")]
pub struct InputGenerationTime(f64);

impl From<f64> for InputGenerationTime {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl InputGenerationTime {
    pub(crate) fn equals(&self, value: f64) -> bool {
        self.0 == value
    }
}

impl TryFrom<InputGenerationTime> for GenerationTime {
    type Error = DemesError;
    fn try_from(value: InputGenerationTime) -> Result<Self, Self::Error> {
        let rv = Self(value.0);
        rv.validate(DemesError::GraphError)?;
        Ok(rv)
    }
}

impl TimeInterval {
    fn contains<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();
        self.start_time > time && time >= self.end_time
    }

    pub(crate) fn new(start_time: Time, end_time: Time) -> Self {
        Self {
            start_time,
            end_time,
        }
    }

    // true if other is in (start_time, end_time]
    pub(crate) fn contains_inclusive_start_exclusive_end<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();

        time > self.end_time && time <= self.start_time
    }

    pub(crate) fn contains_exclusive_start_inclusive_end<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();

        time >= self.end_time && time < self.start_time
    }

    pub(crate) fn contains_inclusive<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();
        self.start_time >= time && time >= self.end_time
    }

    pub(crate) fn duration_greater_than_zero(&self) -> bool {
        self.start_time() > self.end_time()
    }

    pub(crate) fn contains_start_time(&self, other: Time) -> bool {
        self.contains(other)
    }

    /// Return the resolved start time (past) of the interval.
    pub fn start_time(&self) -> Time {
        self.start_time
    }

    /// Return the resolved end time (present) of the interval.
    pub fn end_time(&self) -> Time {
        self.end_time
    }

    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        self.start_time() > other.end_time() && other.start_time() > self.end_time()
    }
}

impl TryFrom<TimeTrampoline> for Time {
    type Error = DemesError;

    fn try_from(value: TimeTrampoline) -> Result<Self, Self::Error> {
        match value {
            // Handle string inputs
            TimeTrampoline::Infinity(string) => {
                if &string == "Infinity" {
                    Ok(Self(f64::INFINITY))
                } else {
                    Err(DemesError::GraphError(string))
                }
            }
            // Fall back to valid YAML representations
            TimeTrampoline::Float(f) => Ok(Self::try_from(f)?),
        }
    }
}

impl From<Time> for TimeTrampoline {
    fn from(value: Time) -> Self {
        if value.0.is_infinite() {
            Self::Infinity("Infinity".to_string())
        } else {
            Self::Float(f64::from(value))
        }
    }
}

impl TryFrom<TimeTrampoline> for InputTime {
    type Error = DemesError;

    fn try_from(value: TimeTrampoline) -> Result<Self, Self::Error> {
        match value {
            // Handle string inputs
            TimeTrampoline::Infinity(string) => {
                if &string == "Infinity" {
                    Ok(Self(f64::INFINITY))
                } else {
                    Err(DemesError::GraphError(string))
                }
            }
            // Fall back to valid YAML representations
            TimeTrampoline::Float(f) => Ok(Self::from(f)),
        }
    }
}

impl std::hash::Hash for HashableTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let value = f64::from(self.0);
        value.to_bits().hash(state)
    }
}

impl PartialEq for HashableTime {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for HashableTime {}

impl From<Time> for HashableTime {
    fn from(time: Time) -> Self {
        Self(time)
    }
}

impl From<HashableTime> for Time {
    fn from(value: HashableTime) -> Self {
        value.0
    }
}

impl From<HashableTime> for f64 {
    fn from(value: HashableTime) -> Self {
        f64::from(value.0)
    }
}

impl From<String> for TimeUnits {
    fn from(value: String) -> Self {
        if &value == "generations" {
            Self::Generations
        } else if &value == "years" {
            Self::Years
        } else {
            Self::Custom(value)
        }
    }
}

impl From<TimeUnits> for String {
    fn from(value: TimeUnits) -> Self {
        value.to_string()
    }
}

impl std::fmt::Display for TimeUnits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnits::Generations => write!(f, "generations"),
            TimeUnits::Years => write!(f, "years"),
            TimeUnits::Custom(custom) => write!(f, "{}", &custom),
        }
    }
}

#[cfg(test)]
mod test_infinity {
    use super::*;

    #[test]
    fn test_infinity_dot_inf() {
        let yaml = "---\n.inf\n";
        let time: Time = serde_yaml::from_str(yaml).unwrap();
        assert!(f64::from(time).is_infinite());
        assert!(f64::from(time).is_sign_positive());
        let yaml = serde_yaml::to_string(&time).unwrap();
        assert!(yaml.contains("Infinity"));
    }

    #[test]
    fn test_infinity_string() {
        let yaml = "---\nInfinity\n";
        let time: Time = serde_yaml::from_str(yaml).unwrap();
        assert!(f64::from(time).is_infinite());
        assert!(f64::from(time).is_sign_positive());
    }
}
