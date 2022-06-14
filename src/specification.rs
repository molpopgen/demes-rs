//! Implement the demes technical
//! [specification](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html)
//! in terms of rust structs.

use crate::DemesError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::Display;
use std::io::Read;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "TimeTrampoline")]
#[serde(into = "TimeTrampoline")]
pub struct Time(f64);

impl From<f64> for Time {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Time {
    fn default_deme_start_time() -> Self {
        Self(f64::INFINITY)
    }

    fn default_epoch_end_time() -> Self {
        Self(0.0)
    }

    fn is_valid_deme_start_time(&self) -> bool {
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

    fn is_valid_epoch_end_time(&self) -> bool {
        self.0.is_finite()
    }

    fn err_if_not_valid_epoch_end_time(&self) -> Result<(), DemesError> {
        if self.is_valid_epoch_end_time() {
            Ok(())
        } else {
            let msg = format!("end_time must be <= t < Infinity, got: {}", self.0);
            Err(DemesError::EpochError(msg))
        }
    }

    fn is_valid_pulse_time(&self) -> bool {
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

impl_newtype_traits!(Time);

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum TimeTrampoline {
    Infinity(String),
    Float(f64),
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
            TimeTrampoline::Float(f) => Ok(Self::from(f)),
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

#[repr(transparent)]
pub struct HashableTime(Time);

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
            let msg = format!("deme sizes must be 0 <= d < Infinity, got: {}", value);
            Err(DemesError::EpochError(msg))
        } else {
            Ok(Self(value))
        }
    }
}

impl_newtype_traits!(DemeSize);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct Proportion(f64);

impl TryFrom<f64> for Proportion {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value <= 0.0 || value > 1.0 {
            let msg = format!(
                "ancestor proportions must be 0.0 < p <= 1.0, got: {}",
                value
            );
            Err(DemesError::DemeError(msg))
        } else {
            Ok(Self(value))
        }
    }
}

impl_newtype_traits!(Proportion);

#[derive(Clone, Copy, Debug)]
pub struct TimeInterval {
    start_time: Time,
    end_time: Time,
}

impl TimeInterval {
    fn contains<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();
        self.start_time.0 > time && time >= self.end_time.0
    }

    // true if other is in (start_time, end_time]
    fn contains_inclusive_start_exclusive_end<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();

        time > self.end_time.0 && time <= self.start_time.0
    }

    fn contains_exclusive_start_inclusive_end<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();

        time >= self.end_time.0 && time < self.start_time.0
    }

    fn contains_inclusive<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();
        self.start_time.0 >= time && time >= self.end_time.0
    }

    fn duration_greater_than_zero(&self) -> bool {
        self.start_time() > self.end_time()
    }

    fn contains_start_time(&self, other: Time) -> bool {
        assert!(other.is_valid_deme_start_time());
        self.contains(other)
    }

    pub fn start_time(&self) -> Time {
        self.start_time
    }

    pub fn end_time(&self) -> Time {
        self.end_time
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.start_time() > other.end_time() && other.start_time() > self.end_time()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SizeFunction {
    Constant,
    Exponential,
    Linear,
}

impl Display for SizeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            SizeFunction::Constant => "constant",
            SizeFunction::Linear => "linear",
            SizeFunction::Exponential => "exponential",
        };
        write!(f, "{}", value)
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
            let msg = format!("cloning rate must be 0.0 <= C <= 1.0, got: {}", value);
            Err(DemesError::EpochError(msg))
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

impl_newtype_traits!(CloningRate);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct SelfingRate(f64);

impl TryFrom<f64> for SelfingRate {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
            let msg = format!("selfing rate must be 0.0 <= S <= 1.0, got: {}", value);
            Err(DemesError::EpochError(msg))
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

impl_newtype_traits!(SelfingRate);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct MigrationRate(f64);

impl TryFrom<f64> for MigrationRate {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
            let msg = format!("migration rate must be 0.0 <= m <= 1.0, got: {value}");
            Err(DemesError::MigrationError(msg))
        } else {
            Ok(Self(value))
        }
    }
}

impl Default for MigrationRate {
    fn default() -> Self {
        Self::try_from(0.0).unwrap()
    }
}

impl_newtype_traits!(MigrationRate);

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedMigration {
    pub demes: Option<Vec<String>>,
    pub source: Option<String>,
    pub dest: Option<String>,
    pub start_time: Option<Time>,
    pub end_time: Option<Time>,
    pub rate: Option<MigrationRate>,
}

impl UnresolvedMigration {
    fn valid_asymmetric_or_err(&self) -> Result<(), DemesError> {
        let mut msg = Option::<String>::default();

        if self.source.is_none() {
            msg = Some("source is None".to_string());
        }
        if self.dest.is_none() {
            msg = Some("dest is None".to_string());
        }
        if self.rate.is_none() {
            msg = Some(format!(
                "rate from source: {} to dest: {} is None",
                self.source.as_ref().unwrap(),
                self.dest.as_ref().unwrap(),
            ));
        };

        match msg {
            Some(message) => Err(DemesError::MigrationError(message)),
            None => Ok(()),
        }
    }

    fn valid_symmetric_or_err(&self) -> Result<(), DemesError> {
        let mut msg = Option::<String>::default();
        if self.demes.is_none() {
            msg = Some("demes is None".to_string());
        }
        if self.rate.is_none() {
            msg = Some(format!(
                "migration rate among {:?} is None",
                self.demes.as_ref().unwrap()
            ));
        };
        match msg {
            Some(message) => Err(DemesError::MigrationError(message)),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AsymmetricMigration {
    source: String,
    dest: String,
    rate: MigrationRate,
    start_time: Option<Time>,
    end_time: Option<Time>,
}

impl AsymmetricMigration {
    fn validate_deme_exists(&self, deme_map: &DemeMap) -> Result<(), DemesError> {
        if !deme_map.contains_key(&self.source) {
            return Err(DemesError::MigrationError(format!(
                "source deme {} is not defined in the graph",
                &self.source
            )));
        }
        if !deme_map.contains_key(&self.dest) {
            return Err(DemesError::MigrationError(format!(
                "dest deme {} is not defined in the graph",
                &self.dest
            )));
        }
        Ok(())
    }

    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn dest(&self) -> &str {
        &self.dest
    }
    pub fn rate(&self) -> MigrationRate {
        self.rate
    }
    pub fn start_time(&self) -> Time {
        self.start_time.unwrap()
    }
    pub fn end_time(&self) -> Time {
        self.end_time.unwrap()
    }
    pub fn time_interval(&self) -> TimeInterval {
        TimeInterval {
            start_time: self.start_time(),
            end_time: self.end_time(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymmetricMigration {
    demes: Vec<String>,
    rate: MigrationRate,
    start_time: Option<Time>,
    end_time: Option<Time>,
}

impl SymmetricMigration {
    fn validate_demes_exists_and_are_unique(&self, deme_map: &DemeMap) -> Result<(), DemesError> {
        if self.demes.len() < 2 {
            return Err(DemesError::MigrationError(
                "the demes field of a migration mut contain at least two demes".to_string(),
            ));
        }
        let mut s = HashSet::<String>::default();
        for name in &self.demes {
            if s.contains(name) {
                return Err(DemesError::MigrationError(format!(
                    "deme name {} present multiple times",
                    name
                )));
            }
            s.insert(name.to_string());
            if !deme_map.contains_key(name) {
                return Err(DemesError::MigrationError(format!(
                    "deme name {} is not defined in the graph",
                    name
                )));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "UnresolvedMigration")]
#[serde(into = "UnresolvedMigration")]
pub enum Migration {
    ASYMMETRIC(AsymmetricMigration),
    SYMMETRIC(SymmetricMigration),
}

impl TryFrom<UnresolvedMigration> for Migration {
    type Error = DemesError;

    fn try_from(value: UnresolvedMigration) -> Result<Self, Self::Error> {
        if value.demes.is_none() {
            if value.source.is_none() || value.dest.is_none() {
                Err(DemesError::MigrationError(
                    "a migration must specify either demes or source and dest".to_string(),
                ))
            } else {
                value.valid_asymmetric_or_err()?;
                Ok(Migration::ASYMMETRIC(AsymmetricMigration {
                    source: value.source.unwrap(),
                    dest: value.dest.unwrap(),
                    rate: value.rate.unwrap(),
                    start_time: value.start_time,
                    end_time: value.end_time,
                }))
            }
        } else if value.source.is_some() || value.dest.is_some() {
            Err(DemesError::MigrationError(
                "a migration must specify either demes or source and dest, but not both"
                    .to_string(),
            ))
        } else {
            value.valid_symmetric_or_err()?;
            Ok(Migration::SYMMETRIC(SymmetricMigration {
                demes: value.demes.unwrap(),
                rate: value.rate.unwrap(),
                start_time: value.start_time,
                end_time: value.end_time,
            }))
        }
    }
}

impl From<Migration> for UnresolvedMigration {
    fn from(value: Migration) -> Self {
        match value {
            Migration::SYMMETRIC(s) => UnresolvedMigration {
                demes: Some(s.demes),
                rate: Some(s.rate),
                start_time: s.start_time,
                end_time: s.end_time,
                source: None,
                dest: None,
            },
            Migration::ASYMMETRIC(a) => UnresolvedMigration {
                demes: None,
                source: Some(a.source),
                dest: Some(a.dest),
                rate: Some(a.rate),
                start_time: a.start_time,
                end_time: a.end_time,
            },
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pulse {
    pub sources: Option<Vec<String>>,
    pub dest: Option<String>,
    pub time: Option<Time>,
    pub proportions: Option<Vec<Proportion>>,
}

impl Pulse {
    fn validate_deme_existence(&self, deme: &str, deme_map: &DemeMap) -> Result<(), DemesError> {
        match deme_map.get(deme) {
            Some(d) => {
                let t = d.time_interval();
                let time = match self.time {
                    Some(t) => t,
                    None => return Err(DemesError::PulseError("time is None".to_string())),
                };
                if !t.contains_inclusive(time) {
                    return Err(DemesError::PulseError(format!(
                        "deme {} does not exist at time of pulse",
                        deme,
                    )));
                }
                Ok(())
            }
            None => Err(DemesError::PulseError(format!(
                "pulse deme {} is invalid",
                deme
            ))),
        }
    }

    fn validate_pulse_time(&self, deme_map: &DemeMap) -> Result<(), DemesError> {
        match self.time {
            Some(time) => {
                if !time.is_valid_pulse_time() {
                    return Err(DemesError::PulseError(format!(
                        "invalid pulse time: {}",
                        time.0
                    )));
                }
            }
            None => return Err(DemesError::PulseError("time is None".to_string())),
        }

        for source_name in self.sources.as_ref().unwrap() {
            let source = deme_map.get(source_name).unwrap();

            let ti = source.time_interval();

            if !ti.contains_exclusive_start_inclusive_end(self.time()) {
                return Err(DemesError::PulseError(format!(
                    "pulse at time: {:?} does not overlap with source: {}",
                    self.time(),
                    source_name
                )));
            }
        }

        let dest = deme_map.get(self.dest()).unwrap();
        let ti = dest.time_interval();
        if !ti.contains_inclusive_start_exclusive_end(self.time()) {
            return Err(DemesError::PulseError(format!(
                "pulse at time: {:?} does not overlap with dest: {}",
                self.time(),
                dest.name(),
            )));
        }

        Ok(())
    }

    fn validate_proportions(&self) -> Result<(), DemesError> {
        if self.proportions.is_none() {
            return Err(DemesError::PulseError("proportions is None".to_string()));
        }
        if self.sources.is_none() {
            return Err(DemesError::PulseError("sources is None".to_string()));
        }

        let proportions = self.proportions.as_ref().unwrap();
        let sources = self.sources.as_ref().unwrap();
        if proportions.len() != sources.len() {
            return Err(DemesError::PulseError(format!("number of sources must equal number of proportions; got {} source and {} proportions", sources.len(), proportions.len())));
        }

        let sum_proportions = proportions
            .iter()
            .fold(0.0, |sum, &proportion| sum + proportion.0);

        if !(1e-9..1.0 + 1e-9).contains(&sum_proportions) {
            return Err(DemesError::PulseError(format!(
                "pulse proportions must sum to 0.0 < p < 1.0, got: {}",
                sum_proportions
            )));
        }

        Ok(())
    }

    fn dest_is_not_source(&self) -> Result<(), DemesError> {
        let dest = self.dest.as_ref().unwrap();
        if self.sources.as_ref().unwrap().contains(dest) {
            Err(DemesError::PulseError(format!(
                "dest: {} is also listed as a source",
                dest
            )))
        } else {
            Ok(())
        }
    }

    fn sources_are_unique(&self) -> Result<(), DemesError> {
        let mut sources = HashSet::<String>::default();
        for source in self.sources.as_ref().unwrap() {
            if sources.contains(source) {
                return Err(DemesError::PulseError(format!(
                    "source: {} listed multiple times",
                    source
                )));
            }
            sources.insert(source.clone());
        }
        Ok(())
    }

    fn validate(&self, deme_map: &DemeMap) -> Result<(), DemesError> {
        self.validate_proportions()?;

        // NOTE: validate proportions is taking care of
        // returning Err if this is not true
        assert!(self.sources.is_some());

        let sources = self.sources.as_ref().unwrap();
        sources
            .iter()
            .try_for_each(|source| self.validate_deme_existence(source, deme_map))?;

        self.dest
            .as_ref()
            .ok_or_else(|| DemesError::PulseError("dest is None".to_string()))?;

        self.validate_deme_existence(self.dest.as_ref().unwrap(), deme_map)?;
        self.dest_is_not_source()?;
        self.sources_are_unique()?;
        self.validate_pulse_time(deme_map)
    }

    fn resolve(&mut self, defaults: &GraphDefaults) -> Result<(), DemesError> {
        defaults.apply_pulse_defaults(self);
        Ok(())
    }

    pub fn time(&self) -> Time {
        match self.time {
            Some(time) => time,
            None => panic!("pulse time is None"),
        }
    }

    pub fn sources(&self) -> &[String] {
        match &self.sources {
            Some(sources) => sources,
            None => panic!("sources are None"),
        }
    }

    pub fn dest(&self) -> &str {
        match &self.dest {
            Some(dest) => dest,
            None => panic!("pulse dest is None"),
        }
    }

    pub fn proportions(&self) -> &[Proportion] {
        match &self.proportions {
            Some(proportions) => proportions,
            None => panic!("proportions are None"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EpochData {
    pub end_time: Option<Time>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    pub start_size: Option<DemeSize>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    pub end_size: Option<DemeSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_function: Option<crate::specification::SizeFunction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloning_rate: Option<crate::specification::CloningRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfing_rate: Option<crate::specification::SelfingRate>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Epoch {
    #[serde(flatten)]
    data: EpochData,
}

impl Epoch {
    fn resolve_size_function(
        &mut self,
        defaults: &GraphDefaults,
        deme_defaults: &DemeDefaults,
    ) -> Result<(), DemesError> {
        if self.data.size_function.is_some() {
            return Ok(());
        }
        match self.data.start_size {
            Some(start_size) => match self.data.end_size {
                Some(end_size) => {
                    if start_size.0 == end_size.0 {
                        self.data.size_function = Some(SizeFunction::Constant);
                    } else {
                        self.data.size_function = defaults.apply_epoch_size_function_defaults(
                            self.data.size_function,
                            deme_defaults,
                        );
                    }
                    Ok(())
                }
                None => Err(DemesError::EpochError("Epoch end_size is None".to_string())),
            },
            None => Err(DemesError::EpochError(
                "Epoch start size is None".to_string(),
            )),
        }
    }

    fn resolve_selfing_rate(&mut self, defaults: &GraphDefaults, deme_defaults: &DemeDefaults) {
        self.data.selfing_rate = match deme_defaults.epoch.selfing_rate {
            Some(selfing_rate) => Some(selfing_rate),
            None => match defaults.epoch.selfing_rate {
                Some(selfing_rate) => Some(selfing_rate),
                None => Some(SelfingRate::default()),
            },
        }
    }

    fn resolve_cloning_rate(&mut self, defaults: &GraphDefaults, deme_defaults: &DemeDefaults) {
        self.data.cloning_rate = match deme_defaults.epoch.cloning_rate {
            Some(cloning_rate) => Some(cloning_rate),
            None => match defaults.epoch.cloning_rate {
                Some(cloning_rate) => Some(cloning_rate),
                None => Some(CloningRate::default()),
            },
        }
    }

    fn resolve(
        &mut self,
        defaults: &GraphDefaults,
        deme_defaults: &DemeDefaults,
    ) -> Result<(), DemesError> {
        self.resolve_selfing_rate(defaults, deme_defaults);
        self.resolve_cloning_rate(defaults, deme_defaults);
        self.resolve_size_function(defaults, deme_defaults)
    }

    fn validate_end_time(&self) -> Result<(), DemesError> {
        match self.data.end_time {
            Some(time) => time.err_if_not_valid_epoch_end_time(),
            None => Err(DemesError::EpochError("end time is None".to_string())),
        }
    }

    fn validate_cloning_rate(&self) -> Result<(), DemesError> {
        match self.data.cloning_rate {
            Some(_) => Ok(()),
            None => Err(DemesError::EpochError("cloning_rate is None".to_string())),
        }
    }

    fn validate_selfing_rate(&self) -> Result<(), DemesError> {
        match self.data.selfing_rate {
            Some(_) => Ok(()),
            None => Err(DemesError::EpochError("selfing_rate is None".to_string())),
        }
    }

    fn validate_size_function(&self) -> Result<(), DemesError> {
        let mut msg: Option<String> = None;

        let start_size = self.data.start_size.unwrap();
        let end_size = self.data.end_size.unwrap();

        match self.data.size_function {
            Some(size_function) => {
                if matches!(size_function, SizeFunction::Constant) {
                    if start_size != end_size {
                        msg = Some(
                            "start_size != end_size paired with size_function: constant"
                                .to_string(),
                        );
                    }
                } else if start_size == end_size {
                    msg = Some(format!(
                "start_size ({:?}) == end_size ({:?}) paired with invalid size_function: {}",
                self.data.start_size, self.data.end_size, size_function
            ));
                }
            }
            None => msg = Some("size_function is None".to_string()),
        }

        match msg {
            Some(error_msg) => Err(DemesError::EpochError(error_msg)),
            None => Ok(()),
        }
    }

    fn validate(&self) -> Result<(), DemesError> {
        self.validate_end_time()?;
        self.validate_cloning_rate()?;
        self.validate_selfing_rate()?;
        self.validate_size_function()
    }

    pub fn size_function(&self) -> SizeFunction {
        self.data.size_function.unwrap()
    }

    pub fn selfing_rate(&self) -> SelfingRate {
        self.data.selfing_rate.unwrap()
    }

    pub fn cloning_rate(&self) -> CloningRate {
        self.data.cloning_rate.unwrap()
    }

    pub fn end_time(&self) -> Time {
        self.data.end_time.unwrap()
    }

    pub fn start_size(&self) -> DemeSize {
        self.data.start_size.unwrap()
    }

    pub fn end_size(&self) -> DemeSize {
        self.data.end_size.unwrap()
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DemeData {
    name: String,
    #[serde(default = "String::default")]
    description: String,
    #[serde(skip)]
    ancestor_map: DemeMap,
    #[serde(default = "Vec::<Epoch>::default")]
    epochs: Vec<Epoch>,
    #[serde(flatten)]
    history: DemeHistory,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemeHistory {
    pub ancestors: Option<Vec<String>>,
    pub proportions: Option<Vec<Proportion>>,
    pub start_time: Option<Time>,
    #[serde(default = "DemeDefaults::default")]
    #[serde(skip_serializing)]
    pub defaults: DemeDefaults,
}

impl PartialEq for DemeData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.description == other.description
            && self.history.ancestors == other.history.ancestors
            && self.history.proportions == other.history.proportions
            && self.history.start_time == other.history.start_time
            && self.epochs == other.epochs
            && self.ancestor_map == other.ancestor_map
    }
}

impl Eq for DemeData {}

pub(crate) type DemePtr = Rc<RefCell<DemeData>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deme(DemePtr);

impl Deme {
    pub(crate) fn new_via_builder(
        name: &str,
        epochs: Vec<EpochData>,
        history: DemeHistory,
        description: Option<&str>,
    ) -> Self {
        let epochs = epochs.into_iter().map(|data| Epoch { data }).collect_vec();
        let description = match description {
            Some(desc) => desc.to_string(),
            None => String::default(),
        };
        let data = DemeData {
            name: name.to_string(),
            epochs,
            history,
            description,
            ..Default::default()
        };
        let ptr = DemePtr::new(RefCell::new(data));
        Self(ptr)
    }

    fn resolve_times(
        &mut self,
        deme_map: &DemeMap,
        defaults: &GraphDefaults,
    ) -> Result<(), DemesError> {
        // apply top-level default if it exists

        {
            let mut mut_borrowed_self = self.0.borrow_mut();
            mut_borrowed_self.history.start_time = match mut_borrowed_self.history.start_time {
                Some(start_time) => Some(start_time),
                None => match defaults.deme.start_time {
                    Some(start_time) => Some(start_time),
                    None => Some(Time::default_deme_start_time()),
                },
            };
        }

        if self
            .0
            .borrow()
            .history
            .ancestors
            .as_ref()
            .unwrap()
            .is_empty()
            && self.start_time() != Time::default_deme_start_time()
        {
            return Err(DemesError::DemeError(format!(
                "deme {} has finite start time but no ancestors",
                self.name()
            )));
        }

        if self.num_ancestors() == 1 {
            let mut mut_borrowed_self = self.0.borrow_mut();

            let ancestors = mut_borrowed_self.history.ancestors.as_ref().unwrap();

            mut_borrowed_self.history.start_time = match mut_borrowed_self.history.start_time {
                Some(start_time) => {
                    if start_time == Time::default_deme_start_time() {
                        Some(
                            deme_map
                                .get(ancestors.get(0).unwrap())
                                .unwrap()
                                .0
                                .borrow() // panic if deme_map doesn't contain name
                                .epochs
                                .last()
                                .unwrap() // panic if ancestor epochs are empty
                                .data
                                .end_time
                                .unwrap(),
                        )
                    } else {
                        Some(start_time)
                    }
                }
                None => Some(Time::default_deme_start_time()),
            };
            //if mut_borrowed_self.start_time == Time::default_deme_start_time() {
            //    mut_borrowed_self.start_time = deme_map
            //        .get(ancestors.get(0).unwrap())
            //        .unwrap()
            //        .0
            //        .borrow() // panic if deme_map doesn't contain name
            //        .epochs
            //        .last()
            //        .unwrap() // panic if ancestor epochs are empty
            //        .end_time
            //        .unwrap();
            match mut_borrowed_self.history.start_time {
                Some(start_time) => match start_time.err_if_not_valid_deme_start_time() {
                    Ok(_) => (),
                    Err(_) => {
                        return Err(DemesError::DemeError(format!(
                            "could not resolve start_time for deme {}",
                            mut_borrowed_self.name
                        )))
                    }
                },
                None => return Err(DemesError::DemeError("start_time is None".to_string())),
            }
        }

        for ancestor in self.0.borrow().history.ancestors.as_ref().unwrap() {
            let a = deme_map.get(ancestor).unwrap();
            let t = a.time_interval();
            if !t.contains_start_time(self.0.borrow().history.start_time.unwrap()) {
                return Err(DemesError::DemeError(format!(
                    "Ancestor {} does not exist at deme {}'s start_time",
                    ancestor,
                    self.name()
                )));
            }
        }

        {
            // last epoch end time defaults to 0,
            // unless defaults are specified
            let mut self_borrow = self.0.borrow_mut();
            // NOTE: cloning the defaults to make borrow checker happy.
            let self_defaults = self_borrow.history.defaults.clone();
            let mut last_epoch_ref = self_borrow.epochs.last_mut().unwrap();
            if last_epoch_ref.data.end_time.is_none() {
                last_epoch_ref.data.end_time = match self_defaults.epoch.end_time {
                    Some(end_time) => Some(end_time),
                    None => match defaults.epoch.end_time {
                        Some(end_time) => Some(end_time),
                        None => Some(Time::default_epoch_end_time()),
                    },
                }
            }
        }

        {
            // apply default epoch start times
            let self_defaults = self.0.borrow().history.defaults.clone();
            for epoch in self.0.borrow_mut().epochs.iter_mut() {
                match epoch.data.end_time {
                    Some(end_time) => end_time.validate(DemesError::EpochError)?,
                    None => {
                        epoch.data.end_time = match self_defaults.epoch.end_time {
                            Some(end_time) => Some(end_time),
                            None => defaults.epoch.end_time,
                        }
                    }
                }
            }
        }

        let mut last_time = f64::from(self.0.borrow().history.start_time.unwrap());
        for (i, epoch) in self.0.borrow().epochs.iter().enumerate() {
            if epoch.data.end_time.is_none() {
                return Err(DemesError::EpochError(format!(
                    "deme: {}, epoch: {} end time must be specified",
                    self.name(),
                    i
                )));
            }
            let end_time = f64::from(epoch.data.end_time.unwrap());
            if end_time >= last_time {
                return Err(DemesError::EpochError(
                    "Epoch end times must be listed in decreasing order".to_string(),
                ));
            }
            last_time = end_time;
            epoch.end_time().validate(DemesError::EpochError)?;
        }

        Ok(())
    }

    fn resolve_first_epoch_sizes(
        &mut self,
        defaults: &GraphDefaults,
    ) -> Result<Option<DemeSize>, DemesError> {
        let mut self_borrow = self.0.borrow_mut();
        let self_defaults = self_borrow.history.defaults.clone();
        let epoch_sizes = {
            let mut temp_epoch = self_borrow.epochs.get_mut(0).unwrap();

            temp_epoch.data.start_size = match temp_epoch.data.start_size {
                Some(start_size) => Some(start_size),
                None => self_defaults.epoch.start_size,
            };
            temp_epoch.data.end_size = match temp_epoch.data.end_size {
                Some(end_size) => Some(end_size),
                None => self_defaults.epoch.end_size,
            };

            defaults.apply_epoch_size_defaults(temp_epoch);
            if temp_epoch.data.start_size.is_none() && temp_epoch.data.end_size.is_none() {
                return Err(DemesError::EpochError(format!(
                    "first epoch of deme {} must define one or both of start_size and end_size",
                    self_borrow.name
                )));
            }
            if temp_epoch.data.start_size.is_none() {
                temp_epoch.data.start_size = temp_epoch.data.end_size;
            }
            if temp_epoch.data.end_size.is_none() {
                temp_epoch.data.end_size = temp_epoch.data.start_size;
            }
            // temp_epoch.clone()
            (temp_epoch.data.start_size, temp_epoch.data.end_size)
        };

        match self_borrow.history.start_time {
            Some(start_time) => {
                if start_time == Time::default_deme_start_time() && epoch_sizes.0 != epoch_sizes.1 {
                    let msg = format!(
                    "first epoch of deme {} cannot have varying size and an infinite time interval: start_size = {}, end_size = {}",
                    self_borrow.name, f64::from(epoch_sizes.0.unwrap()), f64::from(epoch_sizes.1.unwrap()),
                );
                    return Err(DemesError::EpochError(msg));
                }
            }
            None => return Err(DemesError::EpochError("start_time is None".to_string())),
        }
        Ok(epoch_sizes.1)
    }

    fn resolve_sizes(&mut self, defaults: &GraphDefaults) -> Result<(), DemesError> {
        let mut last_end_size = self.resolve_first_epoch_sizes(defaults)?;
        let local_defaults = self.0.borrow().history.defaults.clone();
        for epoch in self.0.borrow_mut().epochs.iter_mut().skip(1) {
            match epoch.data.start_size {
                Some(_) => (),
                None => match local_defaults.epoch.start_size {
                    Some(start_size) => epoch.data.start_size = Some(start_size),
                    None => match defaults.epoch.start_size {
                        Some(start_size) => epoch.data.start_size = Some(start_size),
                        None => epoch.data.start_size = last_end_size,
                    },
                },
            }
            match epoch.data.end_size {
                Some(_) => (),
                None => match local_defaults.epoch.end_size {
                    Some(end_size) => epoch.data.end_size = Some(end_size),
                    None => match defaults.epoch.end_size {
                        Some(end_size) => epoch.data.end_size = Some(end_size),
                        None => epoch.data.end_size = epoch.data.start_size,
                    },
                },
            }
            last_end_size = epoch.data.end_size;
        }
        Ok(())
    }

    fn resolve_proportions(&mut self) -> Result<(), DemesError> {
        let mut borrowed_self = self.0.borrow_mut();

        let num_ancestors = borrowed_self.history.ancestors.as_ref().unwrap().len();
        let proportions = borrowed_self.history.proportions.as_mut().unwrap();

        if proportions.is_empty() && num_ancestors == 1 {
            proportions.push(Proportion(1.0));
        }

        if num_ancestors != proportions.len() {
            return Err(DemesError::DemeError(format!(
                "deme {} ancestors and proportions have different lengths",
                borrowed_self.name
            )));
        }
        Ok(())
    }

    fn check_empty_epochs(&mut self) {
        if self.0.borrow().epochs.is_empty() {
            self.0.borrow_mut().epochs.push(Epoch::default());
        }
    }

    fn apply_toplevel_defaults(&mut self, defaults: &GraphDefaults) {
        let mut borrowed_self = self.0.borrow_mut();
        borrowed_self.history.ancestors = match &borrowed_self.history.ancestors {
            Some(ancestors) => Some(ancestors.to_vec()),
            None => match &defaults.deme.ancestors {
                Some(ancestors) => Some(ancestors.to_vec()),
                None => Some(vec![]),
            },
        };

        borrowed_self.history.proportions = match &borrowed_self.history.proportions {
            Some(proportions) => Some(proportions.to_vec()),
            None => match &defaults.deme.proportions {
                Some(proportions) => Some(proportions.to_vec()),
                None => Some(vec![]),
            },
        };
    }

    fn validate_ancestor_uniqueness(&self, deme_map: &DemeMap) -> Result<(), DemesError> {
        let self_borrow = self.0.borrow();

        let mut msg = Option::<String>::default();
        match &self_borrow.history.ancestors {
            Some(ancestors) => {
                let mut ancestor_set = HashSet::<String>::default();
                for ancestor in ancestors {
                    if ancestor == &self_borrow.name {
                        msg = Some(format!(
                            "deme: {} lists itself as an ancestor",
                            self_borrow.name
                        ));
                    }
                    if !deme_map.contains_key(ancestor) {
                        msg = Some(format!(
                            "deme: {} lists invalid ancestor: {}",
                            self_borrow.name, ancestor
                        ));
                    }
                    if ancestor_set.contains(ancestor) {
                        msg = Some(format!(
                            "deme: {} lists ancestor: {} multiple times",
                            self_borrow.name, ancestor
                        ));
                    }
                    ancestor_set.insert(ancestor.clone());
                }
            }
            None => (),
        }

        match msg {
            None => Ok(()),
            Some(m) => Err(DemesError::DemeError(m)),
        }
    }

    // Make the internal data match the MDM spec
    fn resolve(&mut self, deme_map: &DemeMap, defaults: &GraphDefaults) -> Result<(), DemesError> {
        self.apply_toplevel_defaults(defaults);
        self.validate_ancestor_uniqueness(deme_map)?;
        self.check_empty_epochs();
        assert!(self.0.borrow().ancestor_map.is_empty());
        self.resolve_times(deme_map, defaults)?;
        self.resolve_sizes(defaults)?;
        let self_defaults = self.0.borrow().history.defaults.clone();
        self.0
            .borrow_mut()
            .epochs
            .iter_mut()
            .try_for_each(|e| e.resolve(defaults, &self_defaults))?;
        self.resolve_proportions()?;

        let mut ancestor_map = DemeMap::default();
        let mut mut_self_borrow = self.0.borrow_mut();
        for ancestor in mut_self_borrow.history.ancestors.as_ref().unwrap().iter() {
            ancestor_map.insert(ancestor.clone(), deme_map.get(ancestor).unwrap().clone());
        }
        mut_self_borrow.ancestor_map = ancestor_map;
        Ok(())
    }

    fn validate_start_time(&self) -> Result<(), DemesError> {
        match self.0.borrow().history.start_time {
            Some(start_time) => {
                start_time.validate(DemesError::DemeError)?;
                start_time.err_if_not_valid_deme_start_time()
            }
            None => Err(DemesError::DemeError("start_time is None".to_string())),
        }
    }

    // Names must be valid Python identifiers
    // https://docs.python.org/3/reference/lexical_analysis.html#identifiers
    pub(crate) fn validate_name(&self) -> Result<(), DemesError> {
        let python_identifier = regex::Regex::new(r"^[^\d\W]\w*$").unwrap();
        if python_identifier.is_match(&self.name().to_string()) {
            Ok(())
        } else {
            Err(DemesError::DemeError(format!(
                "invalid deme name: {}:",
                self.name()
            )))
        }
    }

    fn validate(&self) -> Result<(), DemesError> {
        self.validate_name()?;
        self.validate_start_time()?;
        let self_borrow = self.0.borrow();
        if self_borrow.epochs.is_empty() {
            return Err(DemesError::DemeError(format!(
                "no epochs for deme {}",
                self.name()
            )));
        }

        self_borrow.epochs.iter().try_for_each(|e| e.validate())?;

        let proportions = self_borrow.history.proportions.as_ref().unwrap();
        if !proportions.is_empty() {
            let sum_proportions: f64 = proportions.iter().map(|p| f64::from(*p)).sum();
            // NOTE: this is same default as Python's math.isclose().
            if (sum_proportions - 1.0).abs() > 1e-9 {
                return Err(DemesError::DemeError(format!(
                    "proportions for deme {} should sum to ~1.0, got: {}",
                    self_borrow.name, sum_proportions
                )));
            }
        }

        Ok(())
    }

    // Public API

    pub fn time_interval(&self) -> TimeInterval {
        TimeInterval {
            start_time: self.start_time(),
            end_time: self.end_time(),
        }
    }

    pub fn start_time(&self) -> Time {
        self.0.borrow().history.start_time.unwrap()
    }

    pub fn name(&self) -> Ref<'_, String> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| &b.name)
    }

    pub fn num_ancestors(&self) -> usize {
        self.0.borrow().history.ancestors.as_ref().unwrap().len()
    }

    pub fn ancestor_names(&self) -> Ref<'_, [String]> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| match &b.history.ancestors {
            Some(ancestors) => ancestors.as_slice(),
            None => panic!("proportions is None"),
        })
    }

    pub fn description(&self) -> String {
        self.0.borrow().description.clone()
    }

    pub fn num_epochs(&self) -> usize {
        self.0.borrow().epochs.len()
    }

    pub fn epochs(&self) -> Ref<'_, [Epoch]> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| b.epochs.as_slice())
    }

    pub fn proportions(&self) -> Ref<'_, [Proportion]> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| match &b.history.proportions {
            Some(proportions) => proportions.as_slice(),
            None => panic!("proportions is None"),
        })
    }

    pub fn ancestors(&self) -> Ref<'_, DemeMap> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| &b.ancestor_map)
    }

    pub fn start_size(&self) -> DemeSize {
        self.0.borrow().epochs[0].data.start_size.unwrap()
    }

    pub fn end_size(&self) -> DemeSize {
        self.0.borrow().epochs[0].data.end_size.unwrap()
    }

    pub fn start_sizes(&self) -> Vec<DemeSize> {
        self.0
            .borrow()
            .epochs
            .iter()
            .map(|epoch| epoch.start_size())
            .collect()
    }

    pub fn end_sizes(&self) -> Vec<DemeSize> {
        self.0
            .borrow()
            .epochs
            .iter()
            .map(|epoch| epoch.end_size())
            .collect()
    }

    pub fn end_times(&self) -> Vec<Time> {
        self.0
            .borrow()
            .epochs
            .iter()
            .map(|epoch| epoch.end_time())
            .collect()
    }

    pub fn start_times(&self) -> Vec<Time> {
        let mut rv = vec![self.start_time()];
        let end_time = self.end_time();
        let self_borrow = self.0.borrow();

        self_borrow.epochs.iter().for_each(|epoch| {
            let epoch_end = epoch.end_time();
            if epoch_end != end_time {
                rv.push(epoch_end);
            }
        });
        rv
    }

    // Will panic! if the deme is not properly resolved
    pub fn end_time(&self) -> Time {
        self.0
            .borrow()
            .epochs
            .last()
            .as_ref()
            .unwrap()
            .data
            .end_time
            .unwrap()
    }
}

impl PartialEq for Deme {
    fn eq(&self, other: &Self) -> bool {
        let sborrow = self.0.borrow();
        let oborrow = other.0.borrow();
        (*sborrow.deref()).eq(oborrow.deref())
    }
}

impl Eq for Deme {}

type DemeMap = HashMap<String, Deme>;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(from = "String")]
#[serde(into = "String")]
pub enum TimeUnits {
    Generations,
    Years,
    Custom(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[repr(transparent)]
pub struct CustomTimeUnits(String);

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(from = "f64")]
pub struct GenerationTime(f64);

impl From<f64> for GenerationTime {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl GenerationTime {
    fn validate(&self) -> Result<(), DemesError> {
        if !self.0.is_finite() || !self.0.is_sign_positive() || !self.0.gt(&0.0) {
            Err(DemesError::GraphError(format!(
                "generation time must be > 0.0, got: {}",
                self.0
            )))
        } else {
            Ok(())
        }
    }
}

impl_newtype_traits!(GenerationTime);

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphDefaultInput {
    #[serde(flatten)]
    defaults: GraphDefaults,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphDefaults {
    #[serde(default = "EpochData::default")]
    pub epoch: EpochData,
    #[serde(default = "UnresolvedMigration::default")]
    pub migration: UnresolvedMigration,
    #[serde(default = "Pulse::default")]
    pub pulse: Pulse,
    #[serde(default = "TopLevelDemeDefaults::default")]
    pub deme: TopLevelDemeDefaults,
}

impl GraphDefaults {
    fn apply_default_epoch_start_size(&self, start_size: Option<DemeSize>) -> Option<DemeSize> {
        if start_size.is_some() {
            return start_size;
        }
        self.epoch.start_size
    }

    fn apply_default_epoch_end_size(&self, end_size: Option<DemeSize>) -> Option<DemeSize> {
        if end_size.is_some() {
            return end_size;
        }
        self.epoch.end_size
    }

    fn apply_epoch_size_defaults(&self, epoch: &mut Epoch) {
        epoch.data.start_size = self.apply_default_epoch_start_size(epoch.data.start_size);
        epoch.data.end_size = self.apply_default_epoch_end_size(epoch.data.end_size);
    }

    fn apply_epoch_size_function_defaults(
        &self,
        size_function: Option<SizeFunction>,
        deme_level_defaults: &DemeDefaults,
    ) -> Option<SizeFunction> {
        if size_function.is_some() {
            return size_function;
        }

        match deme_level_defaults.epoch.size_function {
            Some(sf) => Some(sf),
            None => match self.epoch.size_function {
                Some(sf) => Some(sf),
                None => Some(SizeFunction::Exponential),
            },
        }
    }

    fn apply_migration_defaults(&self, other: &mut UnresolvedMigration) {
        if other.rate.is_none() {
            other.rate = self.migration.rate;
        }
        if other.start_time.is_none() {
            other.start_time = self.migration.start_time;
        }
        if other.end_time.is_none() {
            other.end_time = self.migration.end_time;
        }
        if other.source.is_none() {
            other.source = self.migration.source.clone();
        }
        if other.dest.is_none() {
            other.dest = self.migration.dest.clone();
        }
        if other.demes.is_none() {
            other.demes = self.migration.demes.clone();
        }
    }

    fn apply_pulse_defaults(&self, other: &mut Pulse) {
        if other.time.is_none() {
            other.time = self.pulse.time;
        }
        if other.sources.is_none() {
            other.sources = self.pulse.sources.clone();
        }
        if other.dest.is_none() {
            other.dest = self.pulse.dest.clone();
        }
        if other.proportions.is_none() {
            other.proportions = self.pulse.proportions.clone();
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopLevelDemeDefaults {
    pub description: Option<String>,
    pub start_time: Option<Time>,
    pub ancestors: Option<Vec<String>>,
    pub proportions: Option<Vec<Proportion>>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemeDefaults {
    pub epoch: EpochData,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Metadata {
    #[serde(flatten)]
    metadata: std::collections::BTreeMap<String, serde_yaml::Value>,
}

impl Metadata {
    fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    pub fn as_yaml_string(&self) -> String {
        serde_yaml::to_string(&self.metadata).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doi: Option<Vec<String>>,
    #[serde(skip_serializing)]
    #[serde(default = "GraphDefaultInput::default")]
    #[serde(rename = "defaults")]
    input_defaults: GraphDefaultInput,
    #[serde(skip)]
    defaults: GraphDefaults,
    #[serde(default = "Metadata::default")]
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    metadata: Metadata,
    time_units: TimeUnits,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_time: Option<GenerationTime>,
    pub(crate) demes: Vec<Deme>,
    #[serde(default = "Vec::<UnresolvedMigration>::default")]
    #[serde(rename = "migrations")]
    #[serde(skip_serializing)]
    input_migrations: Vec<UnresolvedMigration>,
    #[serde(default = "Vec::<AsymmetricMigration>::default")]
    #[serde(rename = "migrations")]
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Vec::<AsymmetricMigration>::is_empty")]
    resolved_migrations: Vec<AsymmetricMigration>,
    #[serde(default = "Vec::<Pulse>::default")]
    pulses: Vec<Pulse>,
    #[serde(skip)]
    deme_map: DemeMap,
}

// NOTE: the manual implementation
// skips over stuff that's only used by the HDM.
// We are testing equality of the MDM only.
impl PartialEq for Graph {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
            && self.doi == other.doi
            && self.time_units == other.time_units
            && self.generation_time == other.generation_time
            && self.demes == other.demes
            && self.resolved_migrations == other.resolved_migrations
            && self.metadata == other.metadata
            && self.pulses == other.pulses
    }
}

impl Eq for Graph {}

impl Graph {
    pub(crate) fn new(
        time_units: TimeUnits,
        generation_time: Option<GenerationTime>,
        defaults: Option<GraphDefaults>,
    ) -> Self {
        let input_defaults = match defaults {
            Some(defaults) => GraphDefaultInput { defaults },
            None => GraphDefaultInput::default(),
        };
        Self {
            time_units,
            input_defaults,
            generation_time,

            // remaining fields have defaults
            description: Option::<String>::default(),
            doi: Option::<Vec<String>>::default(),
            defaults: GraphDefaults::default(),
            metadata: Metadata::default(),
            demes: Vec::<Deme>::default(),
            input_migrations: Vec::<UnresolvedMigration>::default(),
            resolved_migrations: Vec::<AsymmetricMigration>::default(),
            pulses: Vec::<Pulse>::default(),
            deme_map: DemeMap::default(),
        }
    }

    pub(crate) fn new_from_str(yaml: &'_ str) -> Result<Self, DemesError> {
        let g: Self = serde_yaml::from_str(yaml)?;
        Ok(g)
    }

    pub(crate) fn new_from_reader<T: Read>(reader: T) -> Result<Self, DemesError> {
        let g: Self = serde_yaml::from_reader(reader)?;
        Ok(g)
    }

    pub(crate) fn new_resolved_from_str(yaml: &'_ str) -> Result<Self, DemesError> {
        let mut graph = Self::new_from_str(yaml)?;
        graph.resolve()?;
        graph.validate()?;
        Ok(graph)
    }

    pub(crate) fn new_resolved_from_reader<T: Read>(reader: T) -> Result<Self, DemesError> {
        let mut graph = Self::new_from_reader(reader)?;
        graph.resolve()?;
        graph.validate()?;
        Ok(graph)
    }

    pub(crate) fn add_deme(&mut self, deme: Deme) {
        self.demes.push(deme);
    }

    pub(crate) fn add_migration(
        &mut self,
        demes: Option<Vec<String>>,
        source: Option<String>,
        dest: Option<String>,
        rate: Option<MigrationRate>,
        start_time: Option<Time>,
        end_time: Option<Time>,
    ) {
        self.input_migrations.push(UnresolvedMigration {
            demes,
            source,
            dest,
            rate,
            start_time,
            end_time,
        });
    }

    pub(crate) fn add_pulse(
        &mut self,
        sources: Option<Vec<String>>,
        dest: Option<String>,
        time: Option<Time>,
        proportions: Option<Vec<Proportion>>,
    ) {
        self.pulses.push(Pulse {
            sources,
            dest,
            time,
            proportions,
        });
    }

    fn build_deme_map(&self) -> Result<DemeMap, DemesError> {
        let mut rv = DemeMap::default();

        for deme in &self.demes {
            if rv.contains_key(&deme.name().to_string()) {
                return Err(DemesError::DemeError(format!(
                    "duplicate deme name: {}",
                    deme.name(),
                )));
            }
            rv.insert(deme.name().to_string(), deme.clone());
        }

        Ok(rv)
    }

    fn resolve_asymmetric_migration(&mut self, a: AsymmetricMigration) -> Result<(), DemesError> {
        let mut ac = a;

        let source = self.get_deme_from_name(&ac.source).unwrap();
        let dest = self.get_deme_from_name(&ac.dest).unwrap();
        match ac.start_time {
            Some(_) => (),
            None => {
                ac.start_time = Some(std::cmp::min(source.start_time(), dest.start_time()));
            }
        }

        match ac.end_time {
            Some(_) => (),
            None => {
                ac.end_time = Some(std::cmp::max(source.end_time(), dest.end_time()));
            }
        }

        self.resolved_migrations.push(ac);

        Ok(())
    }

    fn process_input_asymmetric_migration(
        &mut self,
        a: &AsymmetricMigration,
    ) -> Result<(), DemesError> {
        a.validate_deme_exists(&self.deme_map)?;
        self.resolve_asymmetric_migration(a.clone())
    }

    fn process_input_symmetric_migration(
        &mut self,
        s: &SymmetricMigration,
    ) -> Result<(), DemesError> {
        s.validate_demes_exists_and_are_unique(&self.deme_map)?;

        // Each input SymmetricMigration becomes two AsymmetricMigration instances
        for (source_name, dest_name) in s.demes.iter().tuple_combinations() {
            assert_ne!(source_name, dest_name);

            let start_time = s.start_time;
            let end_time = s.end_time;

            let a = AsymmetricMigration {
                source: source_name.to_string(),
                dest: dest_name.to_string(),
                rate: s.rate,
                start_time,
                end_time,
            };
            self.resolve_asymmetric_migration(a)?;
            let a = AsymmetricMigration {
                source: dest_name.to_string(),
                dest: source_name.to_string(),
                rate: s.rate,
                start_time,
                end_time,
            };
            self.resolve_asymmetric_migration(a)?;
        }

        Ok(())
    }

    fn resolve_migrations(&mut self) -> Result<(), DemesError> {
        // NOTE: due to the borrow checker not trusting us, we
        // do the old "swap it out" trick to demonstrate
        // that we are not doing bad things.
        // This object is only mut b/c we need to swap it
        let mut input_migrations: Vec<UnresolvedMigration> = vec![];
        std::mem::swap(&mut input_migrations, &mut self.input_migrations);

        if input_migrations.is_empty() {
            // if there are non-default
            // fields in migration defaults,
            // then we go for it and add a default value
            if self.defaults.migration != UnresolvedMigration::default() {
                input_migrations.push(self.defaults.migration.clone());
            }
        }

        for input_mig in &input_migrations {
            let mut input_mig_clone = input_mig.clone();
            self.defaults.apply_migration_defaults(&mut input_mig_clone);
            let m = Migration::try_from(input_mig_clone)?;
            match m {
                Migration::ASYMMETRIC(a) => self.process_input_asymmetric_migration(&a)?,
                Migration::SYMMETRIC(s) => self.process_input_symmetric_migration(&s)?,
            }
        }

        // The spec states that we can discard the unresolved migration stuff,
        // but we'll swap it back. It does no harm to do so.
        std::mem::swap(&mut input_migrations, &mut self.input_migrations);
        Ok(())
    }

    fn build_migration_epochs(&self) -> HashMap<(String, String), Vec<TimeInterval>> {
        let mut rv = HashMap::<(String, String), Vec<TimeInterval>>::default();

        for migration in &self.resolved_migrations {
            let source = migration.source().to_string();
            let dest = migration.dest().to_string();
            let key = (source, dest);

            match rv.get_mut(&key) {
                Some(v) => v.push(migration.time_interval()),
                None => {
                    let _ = rv.insert(key, vec![migration.time_interval()]);
                }
            }
        }

        rv
    }

    fn check_migration_epoch_overlap(&self) -> Result<(), DemesError> {
        let mig_epochs = self.build_migration_epochs();

        for (demes, epochs) in &mig_epochs {
            if epochs.windows(2).any(|w| w[0].overlaps(&w[1])) {
                return Err(DemesError::MigrationError(format!(
                    "overlapping migration epochs between source: {} and dest: {}",
                    demes.0, demes.1
                )));
            }
        }
        Ok(())
    }

    fn get_non_overlapping_migration_intervals(&self) -> Vec<TimeInterval> {
        let mut unique_times = HashSet::<HashableTime>::default();
        for migration in &self.resolved_migrations {
            unique_times.insert(HashableTime(migration.start_time()));
            unique_times.insert(HashableTime(migration.end_time()));
        }
        unique_times.retain(|t| f64::from(t.0).is_finite());

        let mut end_times = unique_times.into_iter().map(|x| x.0).collect::<Vec<_>>();

        // REVERSE sort
        end_times.sort_by(|a, b| b.cmp(a));

        let mut start_times = vec![Time(f64::INFINITY)];

        if let Some((_last, elements)) = end_times.split_last() {
            start_times.extend_from_slice(elements);
        }

        start_times
            .into_iter()
            .zip(end_times.into_iter())
            .map(|times| TimeInterval {
                start_time: times.0,
                end_time: times.1,
            })
            .collect::<Vec<_>>()
    }

    fn validate_input_migration_rates(&self) -> Result<(), DemesError> {
        let intervals = self.get_non_overlapping_migration_intervals();
        let mut input_rates = HashMap::<String, Vec<f64>>::default();

        for deme in self.deme_map.keys() {
            input_rates.insert(deme.clone(), vec![0.0; intervals.len()]);
        }

        for (i, ti) in intervals.iter().enumerate() {
            for migration in &self.resolved_migrations {
                let mti = migration.time_interval();
                if ti.overlaps(&mti) {
                    match input_rates.get_mut(migration.dest()) {
                        Some(rates) => {
                            let rate = rates[i] + migration.rate().0;
                            if rate > 1.0 + 1e-9 {
                                let msg = format!("migration rate into dest: {} is > 1 in the time interval ({:?}, {:?}]",
                                                  migration.dest(), ti.start_time, ti.end_time);
                                return Err(DemesError::MigrationError(msg));
                            }
                            rates[i] = rate;
                        }
                        None => panic!("fatal error when validating migration rate sums"),
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_migrations(&self) -> Result<(), DemesError> {
        for m in &self.resolved_migrations {
            let source = self.get_deme_from_name(&m.source).unwrap();
            let dest = self.get_deme_from_name(&m.dest).unwrap();

            if *source.name() == *dest.name() {
                return Err(DemesError::MigrationError(format!(
                    "source: {} == dest: {}",
                    source.name(),
                    dest.name()
                )));
            }

            match m.start_time {
                None => {
                    return Err(DemesError::MigrationError(format!(
                        "invalid start_time: {:?} for migration between source: {} and dest: {}",
                        m.start_time,
                        source.name(),
                        dest.name()
                    )))
                }
                Some(start_time) => {
                    let interval = source.time_interval();
                    if !interval.contains_inclusive_start_exclusive_end(start_time) {
                        return Err(DemesError::MigrationError(format!(
                            "migration start_time: {:?} does not overlap with existence of source deme {}",
                            start_time,
                            source.name()
                        )));
                    }
                    let interval = dest.time_interval();
                    if !interval.contains_inclusive_start_exclusive_end(start_time) {
                        return Err(DemesError::MigrationError(format!(
                            "migration start_time: {:?} does not overlap with existence of dest deme {}",
                            start_time,
                            dest.name()
                        )));
                    }
                }
            }
            match m.end_time {
                None => {
                    return Err(DemesError::MigrationError(format!(
                        "invalid end_time: {:?} for migration between source: {} and dest: {}",
                        m.end_time,
                        source.name(),
                        dest.name()
                    )))
                }
                Some(end_time) => {
                    if !end_time.0.is_finite() {
                        return Err(DemesError::MigrationError(format!(
                            "invalid migration end_time: {:?}",
                            end_time
                        )));
                    }
                    let interval = source.time_interval();
                    if !interval.contains_exclusive_start_inclusive_end(end_time) {
                        return Err(DemesError::MigrationError(format!(
                            "migration end_time: {:?} does not overlap with existence of source deme {}",
                            end_time,
                            source.name()
                        )));
                    }
                    let interval = dest.time_interval();
                    if !interval.contains_exclusive_start_inclusive_end(end_time) {
                        return Err(DemesError::MigrationError(format!(
                            "migration end_time: {:?} does not overlap with existence of dest deme {}",
                            end_time,
                            dest.name()
                        )));
                    }
                }
            }
            let interval = m.time_interval();
            if !interval.duration_greater_than_zero() {
                return Err(DemesError::MigrationError(format!(
                    "invalid migration duration: {:?} ",
                    interval
                )));
            }
        }
        self.check_migration_epoch_overlap()?;
        self.validate_input_migration_rates()?;
        Ok(())
    }

    fn resolve_pulses(&mut self) -> Result<(), DemesError> {
        if self.pulses.is_empty() && self.defaults.pulse != Pulse::default() {
            self.pulses.push(self.defaults.pulse.clone());
        }
        self.pulses
            .iter_mut()
            .try_for_each(|pulse| pulse.resolve(&self.defaults))?;
        // NOTE: the sort_by flips the order to b, a
        // to put more ancient events at the front.
        self.pulses
            .sort_by(|a, b| b.time.partial_cmp(&a.time).unwrap());
        Ok(())
    }

    pub(crate) fn resolve(&mut self) -> Result<(), DemesError> {
        if self.demes.is_empty() {
            return Err(DemesError::DemeError(
                "no demes have been specified".to_string(),
            ));
        }
        std::mem::swap(&mut self.defaults, &mut self.input_defaults.defaults);
        self.deme_map = self.build_deme_map()?;

        self.demes
            .iter_mut()
            .try_for_each(|deme| deme.resolve(&self.deme_map, &self.defaults))?;
        self.demes.iter().try_for_each(|deme| deme.validate())?;
        self.resolve_migrations()?;
        self.resolve_pulses()?;
        self.validate_migrations()?;
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<(), DemesError> {
        if self.demes.is_empty() {
            return Err(DemesError::DemeError("no demes specified".to_string()));
        }

        if !matches!(&self.time_units, TimeUnits::Generations) && self.generation_time.is_none() {
            return Err(DemesError::GraphError(
                "missing generation_time".to_string(),
            ));
        }

        match self.generation_time {
            None => (),
            Some(value) => value.validate()?,
        }

        if matches!(&self.time_units, TimeUnits::Generations) {
            match self.generation_time {
                Some(value) => {
                    if !value.0.eq(&1.0) {
                        return Err(DemesError::GraphError(
                            "time units are generations but generation_time != 1.0".to_string(),
                        ));
                    }
                }
                None => (),
            }
        }
        self.pulses
            .iter()
            .try_for_each(|pulse| pulse.validate(&self.deme_map))?;

        Ok(())
    }

    pub fn num_demes(&self) -> usize {
        self.demes.len()
    }

    pub fn get_deme_from_name(&self, name: &str) -> Option<&Deme> {
        self.deme_map.get(name)
    }

    pub fn deme(&self, at: usize) -> Deme {
        self.demes[at].clone()
    }

    pub fn demes(&self) -> &[Deme] {
        &self.demes
    }

    pub fn generation_time(&self) -> Option<GenerationTime> {
        self.generation_time
    }

    pub fn time_units(&self) -> TimeUnits {
        self.time_units.clone()
    }
    pub fn migrations(&self) -> &[AsymmetricMigration] {
        &self.resolved_migrations
    }

    pub fn pulses(&self) -> &[Pulse] {
        &self.pulses
    }

    pub fn metadata(&self) -> Option<Metadata> {
        if self.metadata.metadata.is_empty() {
            None
        } else {
            Some(self.metadata.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(sf, SizeFunction::Exponential));

        let yaml = "---\nconstant\n".to_string();
        let sf: SizeFunction = serde_yaml::from_str(&yaml).unwrap();
        assert!(matches!(sf, SizeFunction::Constant));
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

    #[test]
    fn test_epoch_using_defaults() {
        let yaml = "---\nend_time: 1000\nend_size: 100\n".to_string();
        let e: Epoch = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(e.data.end_size.as_ref().unwrap().0, 100.0);
        assert_eq!(e.data.end_time.unwrap().0, 1000.0);
        assert!(e.data.start_size.is_none());
    }

    #[test]
    #[should_panic]
    fn epoch_infinite_end_time() {
        let yaml = "---\nend_time: .inf\nend_size: 100\n".to_string();
        let e: Epoch = serde_yaml::from_str(&yaml).unwrap();
        e.validate_end_time().unwrap();
    }

    #[test]
    #[should_panic]
    fn epoch_infinite_end_size() {
        let yaml = "---\nend_time: 100.3\nend_size: .inf\n".to_string();
        let _: Epoch = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn epoch_bad_size_function() {
        let yaml = "---\nend_time: 100.3\nend_size: 250\nsize_function: ice cream".to_string();
        let _: Epoch = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn epoch_bad_cloning_rate() {
        let yaml =
            "---\nend_time: 100.3\nend_size: 250\nsize_function: exponential\ncloning_rate: -0.0"
                .to_string();
        let _: Epoch = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn epoch_bad_selfing_rate() {
        let yaml =
            "---\nend_time: 100.3\nend_size: 250\nsize_function: constant\nselfing_rate: 1.01"
                .to_string();
        let _: Epoch = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    #[should_panic]
    fn epoch_invalid_field() {
        let yaml = "---\nstart_time: 1000\nend_time: 100.3\nend_size: 250\nsize_function: constant"
            .to_string();
        let e: Epoch = serde_yaml::from_str(&yaml).unwrap();
        println!("{}", serde_yaml::to_string(&e).unwrap());
    }

    #[test]
    fn load_deme_with_two_epochs() {
        let yaml = "---\nname: A great deme!\nepochs:\n - start_size: 500\n   end_time: 500\n - start_size: 200\n   end_size: 100\n".to_string();
        let d: Deme = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(*d.name(), "A great deme!".to_string());
        assert!(d.description().is_empty());
        assert_eq!(d.num_epochs(), 2);
    }

    #[test]
    fn load_deme_with_two_epochs_no_start_size() {
        let yaml = "---\nname: A great deme!\nepochs:\n - end_time: 500\n - start_size: 200\n   end_size: 100\n".to_string();
        let d: Deme = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(*d.name(), "A great deme!".to_string());
        assert!(d.description().is_empty());
        assert_eq!(d.num_epochs(), 2);
    }

    #[test]
    fn load_deme_with_two_ancestors() {
        let yaml = "---\nname: A great deme!\nancestors: [Eleven, ApplePie]".to_string();
        let d: Deme = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(*d.name(), "A great deme!".to_string());
        assert_eq!(d.num_ancestors(), 2);
    }

    #[test]
    fn load_deme_with_two_proportions() {
        let yaml = "---\nname: A great deme!\nproportions: [0.5, 0.5]".to_string();
        let d: Deme = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(*d.name(), "A great deme!".to_string());
        assert_eq!(d.proportions().len(), 2);
        assert!(d.proportions().iter().all(|p| p.0 == 0.5));
    }

    #[test]
    #[should_panic]
    fn load_deme_with_invalid_proportions() {
        let yaml = "---\nname: A great deme!\nproportions: [0.0, 0.5]".to_string();
        let d: Deme = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(&*d.name(), "A great deme!");
        assert_eq!(d.proportions().len(), 2);
        assert!(d.proportions().iter().all(|p| p.0 == 0.5));
    }

    #[test]
    fn test_display() {
        let t = Time::try_from(1.0).unwrap();
        let f = format!("{}", t);
        assert!(f.contains("Time("));
    }

    #[test]
    fn test_time_validity() {
        let t = Time::from(f64::NAN);

        match t.validate(DemesError::DemeError) {
            Ok(_) => (),
            Err(e) => assert!(matches!(e, DemesError::DemeError(_))),
        }
    }

    #[test]
    fn test_newtype_compare_to_f64() {
        {
            let v = Time::from(100.0);
            assert_eq!(v, 100.0);
            assert_eq!(100.0, v);
            assert!(v > 50.0);
            assert!(50.0 < v);
        }

        {
            let v = DemeSize::try_from(100.0).unwrap();
            assert_eq!(v, 100.0);
            assert_eq!(100.0, v);
        }

        {
            let v = SelfingRate::try_from(1.0).unwrap();
            assert_eq!(v, 1.0);
            assert_eq!(1.0, v);
        }

        {
            let v = CloningRate::try_from(1.0).unwrap();
            assert_eq!(v, 1.0);
            assert_eq!(1.0, v);
        }

        {
            let v = Proportion::try_from(1.0).unwrap();
            assert_eq!(v, 1.0);
            assert_eq!(1.0, v);
        }

        {
            let v = MigrationRate::try_from(1.0).unwrap();
            assert_eq!(v, 1.0);
            assert_eq!(1.0, v);
        }
    }
}

#[cfg(test)]
mod test_newtype_ordering {
    use super::*;

    #[test]
    fn test_start_time() {
        let s = Time::try_from(1e-3).unwrap();
        let sd = Time::default_deme_start_time();
        assert!(s < sd);
    }

    #[test]
    #[should_panic]
    fn test_fraud_with_start_time() {
        let s = Time::try_from(1e-3).unwrap();
        let sd = Time(f64::NAN);
        let _ = s < sd;
    }
}

#[cfg(test)]
mod test_graph {
    use super::*;

    #[test]
    fn test_round_trip_with_default_epoch_sizes() {
        let yaml = "
time_units: generations
defaults:
  epoch:
    start_size: 1000
demes:
  - name: A
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.num_demes(), 1);

        // Defaults are part of the HDM and not the MDM.
        // Thus, writing the Graph to YAML should NOT
        // contain that block.
        let y = serde_yaml::to_string(&g).unwrap();
        assert!(!y.contains("defaults:"));

        let _ = Graph::new_resolved_from_str(&y).unwrap();
    }

    #[test]
    fn custom_time_unit_serialization() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  epoch:
    start_size: 1000
demes:
  - name: A
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.num_demes(), 1);

        let y = serde_yaml::to_string(&g).unwrap();
        let _ = Graph::new_resolved_from_str(&y).unwrap();
    }

    #[test]
    fn deserialize_migration_defaults() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  migration:
    rate: 0.25
    source: A
    dest: B
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 42
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.migrations().len(), 1);
        assert_eq!(g.migrations()[0].source(), "A");
        assert_eq!(g.migrations()[0].dest(), "B");
        assert_eq!(f64::from(g.migrations()[0].rate()), 0.25);
    }

    #[test]
    fn deserialize_migration_defaults_rate_only() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  migration:
    rate: 0.25
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 42
migrations:
  - source: A
    dest: B
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.migrations().len(), 1);
        assert_eq!(g.migrations()[0].source(), "A");
        assert_eq!(g.migrations()[0].dest(), "B");
        assert_eq!(f64::from(g.migrations()[0].rate()), 0.25);
    }

    #[test]
    fn deserialize_migration_defaults_source_only() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  migration:
    source: A
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 42
migrations:
  - dest: B
    rate: 0.25
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.migrations().len(), 1);
        assert_eq!(g.migrations()[0].source(), "A");
        assert_eq!(g.migrations()[0].dest(), "B");
        assert_eq!(f64::from(g.migrations()[0].rate()), 0.25);
    }

    #[test]
    fn deserialize_migration_defaults_dest_only() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  migration:
    dest: B
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 42
migrations:
  - source: A
    rate: 0.25
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.migrations().len(), 1);
        assert_eq!(g.migrations()[0].source(), "A");
        assert_eq!(g.migrations()[0].dest(), "B");
        assert_eq!(f64::from(g.migrations()[0].rate()), 0.25);
    }

    #[test]
    fn deserialize_migration_defaults_symmetric() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  migration:
    rate: 0.25
    demes: [A, B]
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 42
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.migrations().len(), 2);
        assert_eq!(g.migrations()[0].source(), "A");
        assert_eq!(g.migrations()[0].dest(), "B");
        assert_eq!(g.migrations()[1].source(), "B");
        assert_eq!(g.migrations()[1].dest(), "A");
        assert_eq!(f64::from(g.migrations()[0].rate()), 0.25);
        assert_eq!(f64::from(g.migrations()[1].rate()), 0.25);
    }

    #[test]
    fn deserialize_migration_defaults_symmetric_swap_deme_order() {
        let yaml = "
time_units: years
description: same tests as above, but demes in different order in migration defaults
generation_time: 25
defaults:
  migration:
    rate: 0.25
    demes: [B, A]
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 42
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.migrations().len(), 2);
        assert_eq!(g.migrations()[0].source(), "B");
        assert_eq!(g.migrations()[0].dest(), "A");
        assert_eq!(g.migrations()[1].source(), "A");
        assert_eq!(g.migrations()[1].dest(), "B");
        assert_eq!(f64::from(g.migrations()[0].rate()), 0.25);
        assert_eq!(f64::from(g.migrations()[1].rate()), 0.25);
    }

    #[test]
    fn deserialize_pulse_defaults() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  pulse: {sources: [A], dest: B, proportions: [0.25], time: 100}
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 250 
";
        let g = Graph::new_resolved_from_str(yaml).unwrap();
        assert_eq!(g.pulses().len(), 1);
        assert_eq!(g.pulses()[0].sources(), vec!["A".to_string()]);
        assert_eq!(g.pulses()[0].dest(), "B");
        assert_eq!(
            g.pulses()[0].proportions(),
            vec![Proportion::try_from(0.25).unwrap()]
        );
        assert_eq!(f64::from(g.pulses()[0].time()), 100.0);
    }
}

#[cfg(test)]
mod test_infinity {
    use super::*;

    #[test]
    fn test_infinity_dot_inf() {
        let yaml = "---\n.inf\n";
        let time: Time = serde_yaml::from_str(yaml).unwrap();
        assert!(time.0.is_infinite());
        assert!(time.0.is_sign_positive());
        let yaml = serde_yaml::to_string(&time).unwrap();
        assert!(yaml.contains("Infinity"));
    }

    #[test]
    fn test_infinity_string() {
        let yaml = "---\nInfinity\n";
        let time: Time = serde_yaml::from_str(yaml).unwrap();
        assert!(time.0.is_infinite());
        assert!(time.0.is_sign_positive());
    }
}
