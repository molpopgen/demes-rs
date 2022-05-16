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
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct Time(f64);

impl TryFrom<f64> for Time {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_nan() || value.is_sign_negative() {
            Err(DemesError::TimeError(value))
        } else {
            Ok(Self(value))
        }
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

    fn err_if_not_valid_deme_start_time(&self) -> Result<(), DemesError> {
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
}

impl_newtype_traits!(Time);

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

    fn contains_inclusive<F>(&self, other: F) -> bool
    where
        F: Into<f64>,
    {
        let time = other.into();
        self.start_time.0 >= time && time >= self.end_time.0
    }

    fn contains_start_time(&self, other: Time) -> bool {
        assert!(other.is_valid_deme_start_time());
        self.contains(other)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SizeFunction {
    CONSTANT,
    EXPONENTIAL,
    LINEAR,
}

impl Display for SizeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            SizeFunction::CONSTANT => "constant",
            SizeFunction::LINEAR => "linear",
            SizeFunction::EXPONENTIAL => "exponential",
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

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct UnresolvedMigration {
    demes: Option<Vec<String>>,
    source: Option<String>,
    dest: Option<String>,
    start_time: Option<Time>,
    end_time: Option<Time>,
    rate: Option<MigrationRate>,
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
pub struct Pulse {
    sources: Option<Vec<String>>,
    dest: Option<String>,
    time: Option<Time>,
    proportions: Option<Vec<Proportion>>,
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
                if !t.contains_start_time(time) {
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

    fn validate_pulse_time(&self) -> Result<(), DemesError> {
        match self.time {
            Some(time) => {
                if time.is_valid_pulse_time() {
                    Ok(())
                } else {
                    Err(DemesError::PulseError(format!(
                        "invalid pulse time: {}",
                        time.0
                    )))
                }
            }
            None => Err(DemesError::PulseError("time is None".to_string())),
        }
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

    fn validate(&self, deme_map: &DemeMap) -> Result<(), DemesError> {
        self.validate_pulse_time()?;
        self.validate_proportions()?;

        // NOTE: validate proportions is taking care of
        // returning Err if this is not true
        assert!(self.sources.is_some());

        let sources = self.sources.as_ref().unwrap();
        sources
            .iter()
            .try_for_each(|source| self.validate_deme_existence(source, deme_map))?;

        //NOTE: the last check should enforce this
        assert!(self.dest.is_some());

        self.validate_deme_existence(self.dest.as_ref().unwrap(), deme_map)
    }

    fn resolve(&mut self) -> Result<(), DemesError> {
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
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Epoch {
    end_time: Option<Time>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    start_size: Option<DemeSize>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    end_size: Option<DemeSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_function: Option<SizeFunction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cloning_rate: Option<CloningRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selfing_rate: Option<SelfingRate>,
}

impl Epoch {
    fn resolve_size_function(&mut self, defaults: &GraphDefaults) -> Result<(), DemesError> {
        if self.size_function.is_some() {
            return Ok(());
        }
        match self.start_size {
            Some(start_size) => match self.end_size {
                Some(end_size) => {
                    if start_size.0 == end_size.0 {
                        self.size_function = Some(SizeFunction::CONSTANT);
                    } else {
                        self.size_function =
                            defaults.apply_epoch_size_function_defaults(self.size_function);
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

    fn resolve_selfing_rate(&mut self, defaults: &GraphDefaults) {
        self.selfing_rate = match defaults.epoch.selfing_rate {
            Some(selfing_rate) => Some(selfing_rate),
            None => Some(SelfingRate::default()),
        }
    }

    fn resolve_cloning_rate(&mut self, defaults: &GraphDefaults) {
        self.cloning_rate = match defaults.epoch.cloning_rate {
            Some(cloning_rate) => Some(cloning_rate),
            None => Some(CloningRate::default()),
        };
    }

    fn resolve(&mut self, defaults: &GraphDefaults) -> Result<(), DemesError> {
        self.resolve_selfing_rate(defaults);
        self.resolve_cloning_rate(defaults);
        self.resolve_size_function(defaults)
    }

    fn validate_end_time(&self) -> Result<(), DemesError> {
        match self.end_time {
            Some(time) => time.err_if_not_valid_epoch_end_time(),
            None => Err(DemesError::EpochError("end time is None".to_string())),
        }
    }

    fn validate_cloning_rate(&self) -> Result<(), DemesError> {
        match self.cloning_rate {
            Some(_) => Ok(()),
            None => Err(DemesError::EpochError("cloning_rate is None".to_string())),
        }
    }

    fn validate_selfing_rate(&self) -> Result<(), DemesError> {
        match self.selfing_rate {
            Some(_) => Ok(()),
            None => Err(DemesError::EpochError("selfing_rate is None".to_string())),
        }
    }

    fn validate_size_function(&self) -> Result<(), DemesError> {
        let mut msg: Option<String> = None;

        let start_size = self.start_size.unwrap();
        let end_size = self.end_size.unwrap();

        match self.size_function {
            Some(size_function) => {
                if matches!(size_function, SizeFunction::CONSTANT) {
                    if start_size != end_size {
                        msg = Some(
                            "start_size != end_size paired with size_function: constant"
                                .to_string(),
                        );
                    }
                } else if start_size == end_size {
                    msg = Some(format!(
                "start_size ({:?}) == end_size ({:?}) paired with invalid size_function: {}",
                self.start_size, self.end_size, size_function
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
        self.size_function.unwrap()
    }

    pub fn selfing_rate(&self) -> SelfingRate {
        self.selfing_rate.unwrap()
    }

    pub fn cloning_rate(&self) -> CloningRate {
        self.cloning_rate.unwrap()
    }

    pub fn end_time(&self) -> Time {
        self.end_time.unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemeData {
    name: String,
    #[serde(default = "String::default")]
    description: String,
    #[serde(default = "Vec::<String>::default")]
    ancestors: Vec<String>,
    #[serde(default = "Vec::<Proportion>::default")]
    proportions: Vec<Proportion>,
    #[serde(default = "Time::default_deme_start_time")]
    start_time: Time,
    #[serde(default = "Vec::<Epoch>::default")]
    epochs: Vec<Epoch>,
    #[serde(skip)]
    ancestor_map: DemeMap,
}

impl PartialEq for DemeData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.description == other.description
            && self.ancestors == other.ancestors
            && self.proportions == other.proportions
            && self.start_time == other.start_time
            && self.epochs == other.epochs
            && self.ancestor_map == other.ancestor_map
    }
}

impl Eq for DemeData {}

type DemePtr = Rc<RefCell<DemeData>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deme(DemePtr);

impl Deme {
    // Will panic! if the deme is not properly resolved
    fn end_time(&self) -> Time {
        self.0
            .borrow()
            .epochs
            .last()
            .as_ref()
            .unwrap()
            .end_time
            .unwrap()
    }

    fn resolve_times(
        &mut self,
        deme_map: &DemeMap,
        defaults: &GraphDefaults,
    ) -> Result<(), DemesError> {
        if self.0.borrow().ancestors.is_empty()
            && self.start_time() != Time::default_deme_start_time()
        {
            return Err(DemesError::DemeError(format!(
                "deme {} has finite start time but no ancestors",
                self.name()
            )));
        }

        if self.num_ancestors() == 1 {
            let mut mut_borrowed_self = self.0.borrow_mut();

            if mut_borrowed_self.start_time == Time::default_deme_start_time() {
                mut_borrowed_self.start_time = deme_map
                    .get(mut_borrowed_self.ancestors.get(0).unwrap())
                    .unwrap()
                    .0
                    .borrow() // panic if deme_map doesn't contain name
                    .epochs
                    .last()
                    .unwrap() // panic if ancestor epochs are empty
                    .end_time
                    .unwrap();
                match mut_borrowed_self
                    .start_time
                    .err_if_not_valid_deme_start_time()
                {
                    Ok(_) => (),
                    Err(_) => {
                        return Err(DemesError::DemeError(format!(
                            "could not resolve start_time for deme {}",
                            mut_borrowed_self.name
                        )))
                    }
                }
            }
        }

        for ancestor in &self.0.borrow().ancestors {
            let a = deme_map.get(ancestor).unwrap();
            let t = a.time_interval();
            if !t.contains_start_time(self.0.borrow().start_time) {
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
            let last_epoch_ref = self_borrow.epochs.last_mut().unwrap();
            if last_epoch_ref.end_time.is_none() {
                last_epoch_ref.end_time = match defaults.epoch.end_time {
                    Some(end_time) => Some(end_time),
                    None => Some(Time::default_epoch_end_time()),
                };
            }
        }

        let mut last_time = f64::from(self.0.borrow().start_time);
        for epoch in &self.0.borrow_mut().epochs {
            if epoch.end_time.is_none() {
                return Err(DemesError::EpochError(
                    "Epoch end time must be specified".to_string(),
                ));
            }
            let end_time = f64::from(epoch.end_time.unwrap());
            if end_time >= last_time {
                return Err(DemesError::EpochError(
                    "Epoch end times must be listed in decreasing order".to_string(),
                ));
            }
            last_time = end_time;
        }

        Ok(())
    }

    fn resolve_first_epoch_sizes(
        &mut self,
        defaults: &GraphDefaults,
    ) -> Result<Option<DemeSize>, DemesError> {
        let mut self_borrow = self.0.borrow_mut();
        let epoch_sizes = {
            let mut temp_epoch = self_borrow.epochs.get_mut(0).unwrap();

            defaults.apply_epoch_size_defaults(temp_epoch);
            if temp_epoch.start_size.is_none() && temp_epoch.end_size.is_none() {
                return Err(DemesError::EpochError(format!(
                    "first epoch of deme {} must define one or both of start_size and end_size",
                    self_borrow.name
                )));
            }
            if temp_epoch.start_size.is_none() {
                temp_epoch.start_size = temp_epoch.end_size;
            }
            if temp_epoch.end_size.is_none() {
                temp_epoch.end_size = temp_epoch.start_size;
            }
            // temp_epoch.clone()
            (temp_epoch.start_size, temp_epoch.end_size)
        };
        if self_borrow.start_time == Time::default_deme_start_time()
            && epoch_sizes.0 != epoch_sizes.1
        {
            let msg = format!(
                    "first epoch of deme {} cannot have varying size and an infinite time interval: start_size = {}, end_size = {}",
                    self_borrow.name, f64::from(epoch_sizes.0.unwrap()), f64::from(epoch_sizes.1.unwrap()),
                );
            return Err(DemesError::EpochError(msg));
        }
        Ok(epoch_sizes.1)
    }

    fn resolve_sizes(&mut self, defaults: &GraphDefaults) -> Result<(), DemesError> {
        let mut last_end_size = self.resolve_first_epoch_sizes(defaults)?;
        for epoch in self.0.borrow_mut().epochs.iter_mut().skip(1) {
            match epoch.start_size {
                Some(_) => (),
                None => match defaults.epoch.start_size {
                    Some(start_size) => epoch.start_size = Some(start_size),
                    None => epoch.start_size = last_end_size,
                },
            }
            match epoch.end_size {
                Some(_) => (),
                None => match defaults.epoch.end_size {
                    Some(end_size) => epoch.end_size = Some(end_size),
                    None => epoch.end_size = epoch.start_size,
                },
            }
            last_end_size = epoch.end_size;
        }
        Ok(())
    }

    fn resolve_proportions(&mut self) -> Result<(), DemesError> {
        let mut borrowed_self = self.0.borrow_mut();

        if borrowed_self.proportions.is_empty() && borrowed_self.ancestors.len() == 1 {
            borrowed_self.proportions.push(Proportion(1.0));
        }

        if borrowed_self.ancestors.len() != borrowed_self.proportions.len() {
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

    // Make the internal data match the MDM spec
    fn resolve(&mut self, deme_map: &DemeMap, defaults: &GraphDefaults) -> Result<(), DemesError> {
        self.check_empty_epochs();
        assert!(self.0.borrow().ancestor_map.is_empty());
        self.resolve_times(deme_map, defaults)?;
        self.resolve_sizes(defaults)?;
        self.0
            .borrow_mut()
            .epochs
            .iter_mut()
            .try_for_each(|e| e.resolve(defaults))?;
        self.resolve_proportions()?;

        let mut ancestor_map = DemeMap::default();
        let mut mut_self_borrow = self.0.borrow_mut();
        for ancestor in &mut_self_borrow.ancestors {
            ancestor_map.insert(ancestor.clone(), deme_map.get(ancestor).unwrap().clone());
        }
        mut_self_borrow.ancestor_map = ancestor_map;
        Ok(())
    }

    fn validate_start_time(&self) -> Result<(), DemesError> {
        self.0
            .borrow()
            .start_time
            .err_if_not_valid_deme_start_time()
    }

    fn validate(&self) -> Result<(), DemesError> {
        self.validate_start_time()?;
        let self_borrow = self.0.borrow();
        if self_borrow.epochs.is_empty() {
            return Err(DemesError::DemeError(format!(
                "no epochs for deme {}",
                self.name()
            )));
        }

        self_borrow.epochs.iter().try_for_each(|e| e.validate())?;

        if !self_borrow.proportions.is_empty() {
            let sum_proportions: f64 = self_borrow.proportions.iter().map(|p| f64::from(*p)).sum();
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
        self.0.borrow().start_time
    }

    pub fn name(&self) -> Ref<'_, String> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| &b.name)
    }

    pub fn num_ancestors(&self) -> usize {
        self.0.borrow().ancestors.len()
    }

    pub fn ancestor_names(&self) -> Ref<'_, [String]> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| b.ancestors.as_slice())
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
        Ref::map(borrow, |b| b.proportions.as_slice())
    }

    pub fn ancestors(&self) -> Ref<'_, DemeMap> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| &b.ancestor_map)
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
    GENERATIONS,
    YEARS,
    CUSTOM(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[repr(transparent)]
pub struct CustomTimeUnits(String);

impl From<String> for TimeUnits {
    fn from(value: String) -> Self {
        if &value == "generations" {
            Self::GENERATIONS
        } else if &value == "years" {
            Self::YEARS
        } else {
            Self::CUSTOM(value)
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
            TimeUnits::GENERATIONS => write!(f, "generations"),
            TimeUnits::YEARS => write!(f, "years"),
            TimeUnits::CUSTOM(custom) => write!(f, "{}", &custom),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct GenerationTime(f64);

impl TryFrom<f64> for GenerationTime {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || !value.is_sign_positive() {
            Err(DemesError::GenerationTimeError(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl_newtype_traits!(GenerationTime);

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphDefaults {
    #[serde(default = "Epoch::default")]
    epoch: Epoch,
    #[serde(default = "UnresolvedMigration::default")]
    migration: UnresolvedMigration,
    #[serde(default = "Pulse::default")]
    pulse: Pulse,
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
        epoch.start_size = self.apply_default_epoch_start_size(epoch.start_size);
        epoch.end_size = self.apply_default_epoch_end_size(epoch.end_size);
    }

    fn apply_epoch_size_function_defaults(
        &self,
        size_function: Option<SizeFunction>,
    ) -> Option<SizeFunction> {
        if size_function.is_some() {
            return size_function;
        }

        match self.epoch.size_function {
            Some(sf) => Some(sf),
            None => Some(SizeFunction::EXPONENTIAL),
        }
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
    #[serde(default = "GraphDefaults::default")]
    defaults: GraphDefaults,
    time_units: TimeUnits,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_time: Option<GenerationTime>,
    demes: Vec<Deme>,
    #[serde(default = "Vec::<Migration>::default")]
    #[serde(rename = "migrations")]
    #[serde(skip_serializing)]
    input_migrations: Vec<Migration>,
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
            && self.deme_map == other.deme_map
    }
}

impl Eq for Graph {}

impl Graph {
    pub(crate) fn new_from_str(yaml: &'_ str) -> Result<Self, DemesError> {
        let g: Self = serde_yaml::from_str(yaml)?;
        Ok(g)
    }

    pub(crate) fn new_resolved_from_str(yaml: &'_ str) -> Result<Self, DemesError> {
        let mut graph = Self::new_from_str(yaml)?;
        graph.resolve()?;
        graph.validate()?;
        Ok(graph)
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

            let a = AsymmetricMigration {
                source: source_name.to_string(),
                dest: dest_name.to_string(),
                rate: s.rate,
                start_time: None,
                end_time: None,
            };
            self.resolve_asymmetric_migration(a)?;
            let a = AsymmetricMigration {
                source: dest_name.to_string(),
                dest: source_name.to_string(),
                rate: s.rate,
                start_time: None,
                end_time: None,
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
        let mut input_migrations: Vec<Migration> = vec![];
        std::mem::swap(&mut input_migrations, &mut self.input_migrations);
        for m in &input_migrations {
            match m {
                Migration::ASYMMETRIC(a) => self.process_input_asymmetric_migration(a)?,
                Migration::SYMMETRIC(s) => self.process_input_symmetric_migration(s)?,
            }
        }

        // The spec states that we can discard the unresolved migration stuff,
        // but we'll swap it back. It does no harm to do so.
        std::mem::swap(&mut input_migrations, &mut self.input_migrations);
        Ok(())
    }

    fn validate_migrations(&self) -> Result<(), DemesError> {
        for m in &self.resolved_migrations {
            let source = self.get_deme_from_name(&m.source).unwrap();
            let dest = self.get_deme_from_name(&m.dest).unwrap();

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
                    if !interval.contains_inclusive(start_time) {
                        return Err(DemesError::MigrationError(format!(
                            "migration start_time: {:?} does not overlap with existence of source deme {}",
                            start_time,
                            source.name()
                        )));
                    }
                    let interval = dest.time_interval();
                    if !interval.contains_inclusive(start_time) {
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
                    let interval = source.time_interval();
                    if !interval.contains_inclusive(end_time) {
                        return Err(DemesError::MigrationError(format!(
                            "migration end_time: {:?} does not overlap with existence of source deme {}",
                            end_time,
                            source.name()
                        )));
                    }
                    let interval = dest.time_interval();
                    if !interval.contains_inclusive(end_time) {
                        return Err(DemesError::MigrationError(format!(
                            "migration end_time: {:?} does not overlap with existence of dest deme {}",
                            end_time,
                            dest.name()
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn resolve_pulses(&mut self) -> Result<(), DemesError> {
        self.pulses
            .iter_mut()
            .try_for_each(|pulse| pulse.resolve())?;
        // NOTE: the sort_by flips the order to b, a
        // to put more ancient events at the front.
        self.pulses
            .sort_by(|a, b| b.time.partial_cmp(&a.time).unwrap());
        Ok(())
    }

    pub(crate) fn resolve(&mut self) -> Result<(), DemesError> {
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
        if !matches!(&self.time_units, TimeUnits::GENERATIONS) && self.generation_time.is_none() {
            return Err(DemesError::TopLevelError(
                "missing generation_time".to_string(),
            ));
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
        let t: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        t.start_time.err_if_not_valid_deme_start_time().unwrap();
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
        let t: TimeInterval = serde_yaml::from_str(&yaml).unwrap();
        t.end_time.err_if_not_valid_epoch_end_time().unwrap();
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
        assert!(matches!(sf, SizeFunction::EXPONENTIAL));

        let yaml = "---\nconstant\n".to_string();
        let sf: SizeFunction = serde_yaml::from_str(&yaml).unwrap();
        assert!(matches!(sf, SizeFunction::CONSTANT));
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
        assert_eq!(e.end_size.as_ref().unwrap().0, 100.0);
        assert_eq!(e.end_time.unwrap().0, 1000.0);
        assert!(e.start_size.is_none());
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
        let yaml = "---\nname: A great deme!\nancestors: [11, Apple Pie]".to_string();
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

    // NOTE: eventually these tests should
    // fail as we support the defaults

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
";
        let _ = Graph::new_resolved_from_str(yaml).unwrap();
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
";
        let _ = Graph::new_resolved_from_str(yaml).unwrap();
    }
}
