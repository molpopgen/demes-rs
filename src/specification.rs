//! Implement the demes technical
//! [specification](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html)
//! in terms of rust structs.

use crate::DemesError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::rc::Rc;

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

impl TryFrom<EndTime> for StartTime {
    type Error = DemesError;

    fn try_from(value: EndTime) -> Result<Self, Self::Error> {
        Self::try_from(f64::from(value))
    }
}

impl Default for StartTime {
    fn default() -> Self {
        Self(f64::INFINITY)
    }
}

impl_newtype_traits!(StartTime);

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

impl Default for EndTime {
    fn default() -> Self {
        Self(0.0)
    }
}

impl_newtype_traits!(EndTime);

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

impl_newtype_traits!(DemeSize);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct Proportion(f64);

impl TryFrom<f64> for Proportion {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value <= 0.0 || value > 1.0 {
            Err(DemesError::ProportionError(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl_newtype_traits!(Proportion);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TimeInterval {
    start_time: StartTime,
    end_time: EndTime,
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

    fn contains_start_time(&self, other: StartTime) -> bool {
        self.contains(other)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SizeFunction {
    #[serde(skip)]
    NONE,
    CONSTANT,
    EXPONENTIAL,
    LINEAR,
}

impl Default for SizeFunction {
    fn default() -> Self {
        Self::NONE
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

impl_newtype_traits!(CloningRate);

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

impl_newtype_traits!(SelfingRate);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct MigrationRate(f64);

impl TryFrom<f64> for MigrationRate {
    type Error = DemesError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
            Err(DemesError::MigrationRateError(value))
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnresolvedMigration {
    demes: Option<Vec<String>>,
    source: Option<String>,
    dest: Option<String>,
    start_time: Option<StartTime>,
    end_time: Option<EndTime>,
    rate: MigrationRate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsymmetricMigration {
    source: String,
    dest: String,
    rate: MigrationRate,
    start_time: Option<StartTime>,
    end_time: Option<EndTime>,
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
    pub fn start_time(&self) -> StartTime {
        self.start_time.unwrap()
    }
    pub fn end_time(&self) -> EndTime {
        self.end_time.unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymmetricMigration {
    demes: Vec<String>,
    rate: MigrationRate,
    start_time: Option<StartTime>,
    end_time: Option<EndTime>,
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
                    rate: value.rate,
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
                rate: value.rate,
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
                rate: s.rate,
                start_time: s.start_time,
                end_time: s.end_time,
                source: None,
                dest: None,
            },
            Migration::ASYMMETRIC(a) => UnresolvedMigration {
                demes: None,
                source: Some(a.source),
                dest: Some(a.dest),
                rate: a.rate,
                start_time: a.start_time,
                end_time: a.end_time,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Epoch {
    end_time: Option<EndTime>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    start_size: Option<DemeSize>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    end_size: Option<DemeSize>,
    #[serde(default = "SizeFunction::default")]
    size_function: SizeFunction,
    #[serde(default = "CloningRate::default")]
    cloning_rate: CloningRate,
    #[serde(default = "SelfingRate::default")]
    selfing_rate: SelfingRate,
}

impl Epoch {
    fn resolve(&mut self) -> Result<(), DemesError> {
        if !matches!(self.size_function, SizeFunction::NONE) {
            return Ok(());
        }
        match self.start_size {
            Some(start_size) => match self.end_size {
                Some(end_size) => {
                    if start_size.0 == end_size.0 {
                        self.size_function = SizeFunction::CONSTANT;
                    } else {
                        self.size_function = SizeFunction::EXPONENTIAL;
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

    fn validate(&self) -> Result<(), DemesError> {
        if !matches!(
            self.size_function,
            SizeFunction::CONSTANT | SizeFunction::EXPONENTIAL | SizeFunction::LINEAR
        ) {
            Err(DemesError::EpochError(format!(
                "unknown size_function: {:?}",
                self.size_function
            )))
        } else if self.start_size.as_ref().unwrap() == self.end_size.as_ref().unwrap() {
            if !matches!(self.size_function, SizeFunction::CONSTANT) {
                Err(DemesError::EpochError(format!(
                    "start_size == end_size paired with invalid size_function {:?}",
                    self.size_function
                )))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
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
    #[serde(default = "StartTime::default")]
    start_time: StartTime,
    #[serde(default = "Vec::<Epoch>::default")]
    epochs: Vec<Epoch>,
    #[serde(skip)]
    ancestor_map: DemeMap,
}

type DemePtr = Rc<RefCell<DemeData>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deme(DemePtr);

impl Deme {
    // Will panic! if the deme is not properly resolved
    fn end_time(&self) -> EndTime {
        self.0
            .borrow()
            .epochs
            .last()
            .as_ref()
            .unwrap()
            .end_time
            .unwrap()
    }

    fn resolve_times(&mut self, deme_map: &DemeMap) -> Result<(), DemesError> {
        if self.0.borrow().ancestors.is_empty() && self.start_time() != StartTime::default() {
            return Err(DemesError::DemeError(format!(
                "deme {} has finite start time but no ancestors",
                self.name()
            )));
        }

        if self.num_ancestors() == 1 {
            let mut mut_borrowed_self = self.0.borrow_mut();

            if mut_borrowed_self.start_time == StartTime::default() {
                mut_borrowed_self.start_time = match StartTime::try_from(
                    deme_map
                        .get(mut_borrowed_self.ancestors.get(0).unwrap())
                        .unwrap()
                        .0
                        .borrow() // panic if deme_map doesn't contain name
                        .epochs
                        .last()
                        .unwrap() // panic if ancestor epochs are empty
                        .end_time
                        .unwrap(), // panic if end_time is None
                ) {
                    Ok(start_time) => start_time,
                    // Err if cannot convert
                    Err(_) => {
                        return Err(DemesError::DemeError(format!(
                            "could not resolve start_time for deme {}",
                            mut_borrowed_self.name
                        )))
                    }
                };
            }
        }

        for ancestor in &self.0.borrow().ancestors {
            let a = deme_map.get(ancestor).unwrap();
            let t = a.time_interval();
            if !t.contains_start_time(self.0.borrow().start_time) {
                return Err(DemesError::AncestorError(format!(
                    "Ancestor {} does not exist at deme {}'s start_time",
                    ancestor,
                    self.name()
                )));
            }
        }

        {
            // last epoch end time defaults to 0
            let mut self_borrow = self.0.borrow_mut();
            let last_epoch_ref = self_borrow.epochs.last_mut().unwrap();
            if last_epoch_ref.end_time.is_none() {
                last_epoch_ref.end_time = Some(EndTime::default());
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
        defaults: Option<GraphDefaults>,
    ) -> Result<Option<DemeSize>, DemesError> {
        let mut self_borrow = self.0.borrow_mut();
        let epoch_sizes = {
            let mut temp_epoch = self_borrow.epochs.get_mut(0).unwrap();

            match defaults {
                Some(d) => {
                    d.apply_epoch_defaults(temp_epoch);
                }
                None => (),
            }
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
        if self_borrow.start_time == StartTime::default() && epoch_sizes.0 != epoch_sizes.1 {
            let msg = format!(
                    "first epoch of deme {} cannot have varying size and an infinite time interval: start_size = {}, end_size = {}",
                    self_borrow.name, f64::from(epoch_sizes.0.unwrap()), f64::from(epoch_sizes.1.unwrap()),
                );
            return Err(DemesError::EpochError(msg));
        }
        Ok(epoch_sizes.1)
    }

    fn resolve_sizes(&mut self, defaults: Option<GraphDefaults>) -> Result<(), DemesError> {
        let mut last_end_size = self.resolve_first_epoch_sizes(defaults)?;
        for epoch in self.0.borrow_mut().epochs.iter_mut().skip(1) {
            match defaults {
                Some(d) => d.apply_epoch_defaults(epoch),
                None => {
                    match epoch.start_size {
                        Some(_) => (),
                        None => epoch.start_size = last_end_size,
                    }
                    match epoch.end_size {
                        Some(_) => (),
                        None => epoch.end_size = epoch.start_size,
                    }
                }
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
    fn resolve(
        &mut self,
        deme_map: &DemeMap,
        defaults: Option<GraphDefaults>,
    ) -> Result<(), DemesError> {
        self.check_empty_epochs();
        assert!(self.0.borrow().ancestor_map.is_empty());
        self.resolve_times(deme_map)?;
        self.resolve_sizes(defaults)?;
        self.0
            .borrow_mut()
            .epochs
            .iter_mut()
            .try_for_each(|e| e.resolve())?;
        self.resolve_proportions()?;

        let mut ancestor_map = DemeMap::default();
        let mut mut_self_borrow = self.0.borrow_mut();
        for ancestor in &mut_self_borrow.ancestors {
            ancestor_map.insert(ancestor.clone(), deme_map.get(ancestor).unwrap().clone());
        }
        mut_self_borrow.ancestor_map = ancestor_map;
        Ok(())
    }

    fn validate(&self) -> Result<(), DemesError> {
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

    pub fn start_time(&self) -> StartTime {
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

    pub fn proportions(&self) -> Ref<'_, [Proportion]> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| b.proportions.as_slice())
    }

    pub fn ancestors(&self) -> Ref<'_, DemeMap> {
        let borrow = self.0.borrow();
        Ref::map(borrow, |b| &b.ancestor_map)
    }
}

type DemeMap = HashMap<String, Deme>;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct EpochDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    start_size: Option<DemeSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_size: Option<DemeSize>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct GraphDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<EpochDefaults>,
}

impl GraphDefaults {
    fn apply_default_epoch_start_size(&self, start_size: Option<DemeSize>) -> Option<DemeSize> {
        if start_size.is_some() {
            return start_size;
        }

        match self.epoch {
            Some(epoch_defaults) => epoch_defaults.start_size,
            None => None,
        }
    }

    fn apply_default_epoch_end_size(&self, end_size: Option<DemeSize>) -> Option<DemeSize> {
        if end_size.is_some() {
            return end_size;
        }

        match self.epoch {
            Some(epoch_defaults) => epoch_defaults.end_size,
            None => None,
        }
    }

    fn apply_epoch_defaults(&self, epoch: &mut Epoch) {
        epoch.start_size = self.apply_default_epoch_start_size(epoch.start_size);
        epoch.end_size = self.apply_default_epoch_end_size(epoch.end_size);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doi: Option<Vec<String>>,
    #[serde(skip_serializing)]
    defaults: Option<GraphDefaults>,
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
    #[serde(skip)]
    deme_map: DemeMap,
}

impl Graph {
    pub(crate) fn new_from_str(yaml: &'_ str) -> Result<Self, Box<dyn std::error::Error>> {
        let g: Self = serde_yaml::from_str(yaml)?;
        Ok(g)
    }

    pub(crate) fn new_resolved_from_str(yaml: &'_ str) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub(crate) fn resolve(&mut self) -> Result<(), DemesError> {
        self.deme_map = self.build_deme_map()?;

        self.demes
            .iter_mut()
            .try_for_each(|deme| deme.resolve(&self.deme_map, self.defaults))?;
        self.demes.iter().try_for_each(|deme| deme.validate())?;
        self.resolve_migrations()?;
        self.validate_migrations()?;
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<(), DemesError> {
        if !matches!(&self.time_units, TimeUnits::GENERATIONS) && self.generation_time.is_none() {
            return Err(DemesError::TopLevelError(
                "missing generation_time".to_string(),
            ));
        }

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
        let _: Epoch = serde_yaml::from_str(&yaml).unwrap();
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
        let s = StartTime::try_from(1e-3).unwrap();
        let sd = StartTime::default();
        assert!(s < sd);
    }

    #[test]
    #[should_panic]
    fn test_fraud_with_start_time() {
        let s = StartTime::try_from(1e-3).unwrap();
        let sd = StartTime(f64::NAN);
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

        let g2 = Graph::new_resolved_from_str(&y).unwrap();
        assert!(g2.defaults.is_none());
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
}
