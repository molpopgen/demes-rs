//! Implement the demes technical
//! [specification](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html)
//! in terms of rust structs.

use crate::time::*;
use crate::CloningRate;
use crate::DemeSize;
use crate::DemesError;
use crate::InputCloningRate;
use crate::InputDemeSize;
use crate::InputMigrationRate;
use crate::InputProportion;
use crate::InputSelfingRate;
use crate::MigrationRate;
use crate::Proportion;
use crate::SelfingRate;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::Display;
use std::io::Read;

macro_rules! get_deme {
    ($name: expr, $deme_map: expr, $demes: expr) => {
        match $deme_map.get($name) {
            Some(index) => $demes.get(*index),
            None => None,
        }
    };
}

/// Specify how deme sizes change during an [`Epoch`](crate::Epoch).
///
/// # Examples
///
/// ```
/// let yaml = "
/// time_units: years
/// generation_time: 25
/// description:
///   A deme of 50 individuals that grew to 100 individuals
///   in the last 100 years.
///   Default behavior is that size changes are exponential.
/// demes:
///  - name: deme
///    epochs:
///     - start_size: 50
///       end_time: 100
///     - start_size: 50
///       end_size: 100
/// ";
/// let graph = demes::loads(yaml).unwrap();
/// let deme = graph.get_deme_from_name("deme").unwrap();
/// assert_eq!(deme.num_epochs(), 2);
/// let last_epoch = deme.get_epoch(1).unwrap();
/// assert!(matches!(last_epoch.size_function(),
///                  demes::SizeFunction::Exponential));
/// let first_epoch = deme.get_epoch(0).unwrap();
/// assert!(matches!(first_epoch.size_function(),
///                  demes::SizeFunction::Constant));
/// ```
///
/// Let's change the function to linear for the second
/// epoch:
///
/// ```
/// let yaml = "
/// time_units: years
/// generation_time: 25
/// description:
///   A deme of 50 individuals that grew to 100 individuals
///   in the last 100 years.
/// demes:
///  - name: deme
///    epochs:
///     - start_size: 50
///       end_time: 100
///     - start_size: 50
///       end_size: 100
///       size_function: linear
/// ";
/// let graph = demes::loads(yaml).unwrap();
/// let deme = graph.get_deme_from_name("deme").unwrap();
/// let last_epoch = deme.get_epoch(1).unwrap();
/// assert!(matches!(last_epoch.size_function(),
///                  demes::SizeFunction::Linear));
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SizeFunction {
    #[allow(missing_docs)]
    Constant,
    #[allow(missing_docs)]
    Exponential,
    #[allow(missing_docs)]
    Linear,
}

impl Display for SizeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            SizeFunction::Constant => "constant",
            SizeFunction::Linear => "linear",
            SizeFunction::Exponential => "exponential",
        };
        write!(f, "{value}")
    }
}

/// A deme can be identified as an index
/// or as a name
#[derive(Copy, Clone, Debug)]
pub enum DemeId<'name> {
    /// The index of a deme
    Index(usize),
    /// The name of a deme
    Name(&'name str),
}

impl<'name> From<usize> for DemeId<'name> {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl<'name> From<&'name str> for DemeId<'name> {
    fn from(value: &'name str) -> Self {
        Self::Name(value)
    }
}

/// An unresolved migration epoch.
///
/// All input migrations are resolved to [`AsymmetricMigration`](crate::AsymmetricMigration)
/// instances.
///
/// # Examples
///
/// ## [`GraphBuilder`](crate::GraphBuilder)
///
/// This type supports member field initialization using defaults.
/// This form of initalization is used in:
///
/// * [`GraphDefaults`](crate::GraphDefaults)
///
/// ```
/// let _ = demes::UnresolvedMigration{source: Some("A".to_string()),
///                                    dest: Some("B".to_string()),
///                                    rate: Some(0.2.into()),
///                                    ..Default::default()
///                                    };
/// ```
#[derive(Clone, Default, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedMigration {
    /// The demes involved in symmetric migration epochs
    pub demes: Option<Vec<String>>,
    /// The source deme of an asymmetric migration epoch
    pub source: Option<String>,
    /// The destination deme of an asymmetric migration epoch
    pub dest: Option<String>,
    /// The start time of a migration epoch
    pub start_time: Option<InputTime>,
    /// The end time of a migration epoch
    pub end_time: Option<InputTime>,
    /// The rate during a migration epoch
    pub rate: Option<InputMigrationRate>,
}

impl UnresolvedMigration {
    fn validate(&self) -> Result<(), DemesError> {
        if let Some(value) = self.start_time {
            Time::try_from(value).map_err(|_| {
                DemesError::MigrationError(format!("invalid start_time: {value:?}"))
            })?;
        }
        if let Some(value) = self.end_time {
            Time::try_from(value)
                .map_err(|_| DemesError::MigrationError(format!("invalid end_time: {value:?}")))?;
        }
        Ok(())
    }

    fn valid_asymmetric_or_err(&self) -> Result<(), DemesError> {
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| DemesError::MigrationError("source is none".to_string()))?;

        let dest = self
            .dest
            .as_ref()
            .ok_or_else(|| DemesError::MigrationError("dest is none".to_string()))?;

        self.rate.ok_or_else(|| {
            DemesError::MigrationError(format!(
                "rate frmm source: {source} to dest: {dest} is None",
            ))
        })?;

        Ok(())
    }

    fn valid_symmetric_or_err(&self) -> Result<(), DemesError> {
        let demes = self
            .demes
            .as_ref()
            .ok_or_else(|| DemesError::MigrationError("demes is None".to_string()))?;
        self.rate.ok_or_else(|| {
            DemesError::MigrationError(format!("migration rate among {demes:?} is None",))
        })?;
        Ok(())
    }

    fn resolved_rate_or_err(&self) -> Result<InputMigrationRate, DemesError> {
        self.rate
            .ok_or_else(|| DemesError::MigrationError("migration rate not resolved".to_string()))
    }

    fn resolved_dest_or_err(&self) -> Result<String, DemesError> {
        match &self.dest {
            Some(dest) => Ok(dest.to_string()),
            None => Err(DemesError::MigrationError(
                "migration dest not resolved".to_string(),
            )),
        }
    }

    fn resolved_source_or_err(&self) -> Result<String, DemesError> {
        match &self.source {
            Some(source) => Ok(source.to_string()),
            None => Err(DemesError::MigrationError(
                "migration source not resolved".to_string(),
            )),
        }
    }

    /// Set the source deme
    ///
    /// See ['GraphBuilder'].
    ///
    /// # Examples
    ///
    /// ```
    /// let _ = demes::UnresolvedMigration::default().set_source("A");
    /// ```
    pub fn set_source<A>(self, source: A) -> Self
    where
        A: AsRef<str>,
    {
        Self {
            source: Some(source.as_ref().to_owned()),
            ..self
        }
    }

    /// Set the destination deme
    ///
    /// See ['GraphBuilder'].
    ///
    /// # Examples
    ///
    /// ```
    /// let _ = demes::UnresolvedMigration::default().set_dest("A");
    /// ```
    pub fn set_dest<A>(self, dest: A) -> Self
    where
        A: AsRef<str>,
    {
        Self {
            dest: Some(dest.as_ref().to_owned()),
            ..self
        }
    }

    /// Set the demes
    ///
    /// See ['GraphBuilder'].
    ///
    /// # Examples
    ///
    /// ```
    /// let _ = demes::UnresolvedMigration::default().set_demes(["A", "B"].as_slice());
    /// ```
    pub fn set_demes<I, A>(self, d: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: AsRef<str>,
    {
        Self {
            demes: Some(
                d.into_iter()
                    .map(|a| a.as_ref().to_owned())
                    .collect::<Vec<_>>(),
            ),
            ..self
        }
    }

    /// Set the start time
    ///
    /// See ['GraphBuilder'].
    ///
    /// # Examples
    ///
    /// ```
    /// let _ = demes::UnresolvedMigration::default().set_start_time(1.0);
    /// ```
    pub fn set_start_time<T>(self, time: T) -> Self
    where
        T: Into<InputTime>,
    {
        Self {
            start_time: Some(time.into()),
            ..self
        }
    }

    /// Set the end time
    ///
    /// See ['GraphBuilder'].
    ///
    /// # Examples
    ///
    /// ```
    /// let _ = demes::UnresolvedMigration::default().set_end_time(10.);
    /// ```
    pub fn set_end_time<T>(self, time: T) -> Self
    where
        T: Into<InputTime>,
    {
        Self {
            end_time: Some(time.into()),
            ..self
        }
    }

    /// Set the symmetric migration rate among all `demes`.
    ///
    /// See ['GraphBuilder'].
    ///
    /// # Examples
    ///
    /// ```
    /// let _ = demes::UnresolvedMigration::default().set_rate(0.3333);
    /// ```
    pub fn set_rate<R>(self, rate: R) -> Self
    where
        R: Into<InputMigrationRate>,
    {
        Self {
            rate: Some(rate.into()),
            ..self
        }
    }
}

/// An asymmetric migration epoch.
///
/// All input migrations are resolved to asymmetric migration instances.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct AsymmetricMigration {
    source: String,
    dest: String,
    rate: MigrationRate,
    start_time: Time,
    end_time: Time,
}

impl AsymmetricMigration {
    fn resolved_time_to_generations(
        &mut self,
        generation_time: GenerationTime,
        rounding: fn(Time, GenerationTime) -> Time,
    ) -> Result<(), DemesError> {
        self.start_time = convert_resolved_time_to_generations(
            generation_time,
            rounding,
            DemesError::MigrationError,
            "start_time is not resolved",
            Some(self.start_time),
        )?;
        self.end_time = convert_resolved_time_to_generations(
            generation_time,
            rounding,
            DemesError::MigrationError,
            "end_time is not resolved",
            Some(self.end_time),
        )?;

        if self.end_time >= self.start_time {
            Err(DemesError::MigrationError(
                "conversion of migration times to generations resulted in a zero-length epoch"
                    .to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Get name of the source deme
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get name of the destination deme
    pub fn dest(&self) -> &str {
        &self.dest
    }

    /// Get the resolved migration rate
    pub fn rate(&self) -> MigrationRate {
        self.rate
    }

    /// Resolved start [`Time`](crate::Time) of the migration epoch
    pub fn start_time(&self) -> Time {
        self.start_time
    }

    /// Resolved end [`Time`](crate::Time) of the migration epoch
    pub fn end_time(&self) -> Time {
        self.end_time
    }

    /// Resolved time interval of the migration epoch
    pub fn time_interval(&self) -> TimeInterval {
        TimeInterval::new(self.start_time(), self.end_time())
    }
}

#[derive(Clone, Debug)]
enum Migration {
    Asymmetric(UnresolvedMigration),
    Symmetric(UnresolvedMigration),
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
                Ok(Migration::Asymmetric(UnresolvedMigration {
                    demes: None,
                    source: Some(value.resolved_source_or_err()?),
                    dest: Some(value.resolved_dest_or_err()?),
                    rate: Some(value.resolved_rate_or_err()?),
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
            Ok(Migration::Symmetric(value))
        }
    }
}

impl From<Migration> for UnresolvedMigration {
    fn from(value: Migration) -> Self {
        match value {
            Migration::Symmetric(s) => s,
            Migration::Asymmetric(a) => a,
        }
    }
}

/// A resolved Pulse event
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pulse {
    sources: Vec<String>,
    dest: String,
    time: Time,
    proportions: Vec<Proportion>,
}

/// An unresolved Pulse event.
#[derive(Clone, Default, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedPulse {
    #[allow(missing_docs)]
    pub sources: Option<Vec<String>>,
    #[allow(missing_docs)]
    pub dest: Option<String>,
    #[allow(missing_docs)]
    pub time: Option<InputTime>,
    #[allow(missing_docs)]
    pub proportions: Option<Vec<InputProportion>>,
}

impl TryFrom<UnresolvedPulse> for Pulse {
    type Error = DemesError;
    fn try_from(value: UnresolvedPulse) -> Result<Self, Self::Error> {
        let input_proportions = value.proportions.ok_or_else(|| {
            DemesError::PulseError("pulse proportions are unresolved".to_string())
        })?;
        let mut proportions = vec![];
        for p in input_proportions {
            proportions.push(Proportion::try_from(p)?);
        }
        Ok(Self {
            sources: value.sources.ok_or_else(|| {
                DemesError::PulseError("pulse sources are unresolved".to_string())
            })?,
            dest: value
                .dest
                .ok_or_else(|| DemesError::PulseError("pulse dest are unresolved".to_string()))?,
            time: value
                .time
                .ok_or_else(|| DemesError::PulseError("pulse time are unresolved".to_string()))?
                .try_into()?,
            proportions,
        })
    }
}

impl UnresolvedPulse {
    fn validate_as_default(&self) -> Result<(), DemesError> {
        if let Some(value) = self.time {
            Time::try_from(value)
                .map_err(|_| DemesError::PulseError(format!("invalid time: {value:?}")))?;
        }

        match &self.proportions {
            Some(value) => {
                for v in value {
                    if Proportion::try_from(*v).is_err() {
                        return Err(DemesError::PulseError(format!(
                            "invalid proportion: {:?}",
                            *v
                        )));
                    }
                }
            }
            None => (),
        }

        Ok(())
    }

    fn get_proportions(&self) -> Result<&[InputProportion], DemesError> {
        Ok(self
            .proportions
            .as_ref()
            .ok_or_else(|| DemesError::PulseError("proportions are None".to_string()))?)
    }

    fn get_time(&self) -> Result<Time, DemesError> {
        self.time
            .ok_or_else(|| DemesError::PulseError("time is None".to_string()))?
            .try_into()
    }

    fn get_sources(&self) -> Result<&[String], DemesError> {
        Ok(self
            .sources
            .as_ref()
            .ok_or_else(|| DemesError::PulseError("sources are None".to_string()))?)
    }

    fn get_dest(&self) -> Result<&str, DemesError> {
        Ok(self
            .dest
            .as_ref()
            .ok_or_else(|| DemesError::PulseError("pulse dest is None".to_string()))?)
    }

    fn resolve(&mut self, defaults: &GraphDefaults) -> Result<(), DemesError> {
        defaults.apply_pulse_defaults(self);
        Ok(())
    }

    fn validate_proportions(&self, sources: &[String]) -> Result<(), DemesError> {
        if self.proportions.is_none() {
            return Err(DemesError::PulseError("proportions is None".to_string()));
        }
        let proportions = self.get_proportions()?;
        for p in proportions.iter() {
            Proportion::try_from(*p)
                .map_err(|_| DemesError::PulseError(format!("invalid proportion: {:?}", *p)))?;
        }
        if proportions.len() != sources.len() {
            return Err(DemesError::PulseError(format!("number of sources must equal number of proportions; got {} source and {} proportions", sources.len(), proportions.len())));
        }

        let sum_proportions = proportions
            .iter()
            .fold(0.0, |sum, &proportion| sum + f64::from(proportion));

        if !(1e-9..1.0 + 1e-9).contains(&sum_proportions) {
            return Err(DemesError::PulseError(format!(
                "pulse proportions must sum to 0.0 < p < 1.0, got: {sum_proportions}",
            )));
        }

        Ok(())
    }

    fn validate_pulse_time(
        &self,
        deme_map: &DemeMap,
        demes: &[UnresolvedDeme],
        time: Time,
        dest: &str,
        sources: &[String],
    ) -> Result<(), DemesError> {
        if !time.is_valid_pulse_time() {
            return Err(DemesError::PulseError(format!(
                "invalid pulse time: {}",
                f64::from(time)
            )));
        }

        for source_name in sources {
            let source = get_deme!(source_name, deme_map, demes).ok_or_else(|| {
                DemesError::PulseError(format!("invalid pulse source: {source_name}"))
            })?;

            let ti = source.get_time_interval()?;

            if !ti.contains_exclusive_start_inclusive_end(time) {
                return Err(DemesError::PulseError(format!(
                    "pulse at time: {time:?} does not overlap with source: {source_name}",
                )));
            }
        }

        let dest_deme = get_deme!(dest, deme_map, demes)
            .ok_or_else(|| DemesError::PulseError(format!("invalid pulse dest: {dest}")))?;
        let ti = dest_deme.get_time_interval()?;
        if !ti.contains_inclusive_start_exclusive_end(time) {
            return Err(DemesError::PulseError(format!(
                "pulse at time: {:?} does not overlap with dest: {}",
                time, dest_deme.name,
            )));
        }

        Ok(())
    }

    fn validate_destination_deme_existence(
        &self,
        dest: &str,
        deme_map: &DemeMap,
        demes: &[UnresolvedDeme],
        time: Time,
    ) -> Result<(), DemesError> {
        match get_deme!(dest, deme_map, demes) {
            Some(d) => {
                let t = d.get_time_interval()?;
                if !t.contains_inclusive(time) {
                    return Err(DemesError::PulseError(format!(
                        "destination deme {dest} does not exist at time of pulse",
                    )));
                }
                Ok(())
            }
            None => Err(DemesError::PulseError(format!(
                "pulse deme {dest} is invalid",
            ))),
        }
    }

    fn dest_is_not_source(&self, dest: &str, sources: &[String]) -> Result<(), DemesError> {
        if sources.iter().any(|s| s.as_str() == dest) {
            Err(DemesError::PulseError(format!(
                "dest: {dest} is also listed as a source",
            )))
        } else {
            Ok(())
        }
    }

    fn sources_are_unique(&self, sources: &[String]) -> Result<(), DemesError> {
        let mut unique_sources = HashSet::<String>::default();
        for source in sources {
            if unique_sources.contains(source) {
                return Err(DemesError::PulseError(format!(
                    "source: {source} listed multiple times",
                )));
            }
            unique_sources.insert(source.clone());
        }
        Ok(())
    }

    fn validate(&self, deme_map: &DemeMap, demes: &[UnresolvedDeme]) -> Result<(), DemesError> {
        let dest = self.get_dest()?;
        let sources = self.get_sources()?;
        let time = self.get_time()?;
        self.validate_proportions(sources)?;

        sources.iter().try_for_each(|source| {
            self.validate_destination_deme_existence(source, deme_map, demes, time)
        })?;

        self.validate_destination_deme_existence(dest, deme_map, demes, time)?;
        self.dest_is_not_source(dest, sources)?;
        self.sources_are_unique(sources)?;
        self.validate_pulse_time(deme_map, demes, time, dest, sources)
    }
}

impl Pulse {
    fn resolved_time_to_generations(
        &mut self,
        generation_time: GenerationTime,
        rounding: fn(Time, GenerationTime) -> Time,
    ) -> Result<(), DemesError> {
        self.time = convert_resolved_time_to_generations(
            generation_time,
            rounding,
            DemesError::PulseError,
            "pulse time is note resolved",
            Some(self.time),
        )?;
        Ok(())
    }

    /// Resolved time of the pulse
    pub fn time(&self) -> Time {
        self.time
    }

    /// Resolved pulse source demes as slice
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Resolved pulse destination deme
    pub fn dest(&self) -> &str {
        &self.dest
    }

    /// Resolved pulse proportions
    pub fn proportions(&self) -> &[Proportion] {
        &self.proportions
    }
}

/// HDM representation of an epoch.
///
/// Direct construction of this type is useful in:
/// * [`DemeDefaults`](crate::DemeDefaults)
/// * [`GraphDefaults`](crate::GraphDefaults)
///
/// # Examples
///
/// This type supports field initialization with defaults:
///
/// ```
/// let _ = demes::UnresolvedEpoch{
///              start_size: Some(demes::InputDemeSize::from(1e6)),
///              ..Default::default()
///              };
/// ```
///
/// Type inference improves ergonomics:
///
/// ```
/// let _ = demes::UnresolvedEpoch{
///              start_size: Some(1e6.into()),
///              ..Default::default()
///              };
/// ```
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedEpoch {
    #[allow(missing_docs)]
    pub end_time: Option<InputTime>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    #[allow(missing_docs)]
    pub start_size: Option<InputDemeSize>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    #[allow(missing_docs)]
    pub end_size: Option<InputDemeSize>,
    #[allow(missing_docs)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_function: Option<crate::specification::SizeFunction>,
    #[allow(missing_docs)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloning_rate: Option<InputCloningRate>,
    #[allow(missing_docs)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfing_rate: Option<InputSelfingRate>,
}

impl UnresolvedEpoch {
    fn validate_as_default(&self) -> Result<(), DemesError> {
        if let Some(value) = self.end_time {
            Time::try_from(value)
                .map_err(|_| DemesError::EpochError(format!("invalid end_time: {value:?}")))?;
        }
        if let Some(value) = self.start_size {
            DemeSize::try_from(value)
                .map_err(|_| DemesError::EpochError(format!("invalid start_size: {value:?}")))?;
        }
        if let Some(value) = self.end_size {
            DemeSize::try_from(value)
                .map_err(|_| DemesError::EpochError(format!("invalid end_size: {value:?}")))?;
        }
        if let Some(value) = self.cloning_rate {
            CloningRate::try_from(value)
                .map_err(|_| DemesError::EpochError(format!("invalid cloning_rate: {value:?}")))?;
        }
        if let Some(value) = self.selfing_rate {
            SelfingRate::try_from(value)
                .map_err(|_| DemesError::EpochError(format!("invalid selfing_rate: {value:?}")))?;
        }
        Ok(())
    }
}

/// A resolved epoch
#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Epoch {
    #[serde(skip)]
    start_time: Time,
    end_time: Time,
    start_size: DemeSize,
    end_size: DemeSize,
    size_function: SizeFunction,
    cloning_rate: CloningRate,
    selfing_rate: SelfingRate,
}

impl Epoch {
    fn new_from_unresolved(
        start_time: InputTime,
        unresolved: UnresolvedEpoch,
    ) -> Result<Self, DemesError> {
        Ok(Self {
            start_time: start_time.try_into().map_err(|_| {
                DemesError::EpochError(format!("invalid start_time: {start_time:?}"))
            })?,
            end_time: unresolved
                .end_time
                .ok_or_else(|| DemesError::EpochError("end_time unresolved".to_string()))?
                .try_into()
                .map_err(|_| {
                    DemesError::EpochError(format!("invalid end_time: {:?}", unresolved.end_time))
                })?,
            start_size: unresolved
                .start_size
                .ok_or_else(|| DemesError::EpochError("end_time unresolved".to_string()))?
                .try_into()
                .map_err(|_| {
                    DemesError::EpochError(format!(
                        "invalid start_size: {:?}",
                        unresolved.start_size
                    ))
                })?,
            end_size: unresolved
                .end_size
                .ok_or_else(|| DemesError::EpochError("end_time unresolved".to_string()))?
                .try_into()
                .map_err(|_| {
                    DemesError::EpochError(format!(
                        "invalid cloning_rate: {:?}",
                        unresolved.cloning_rate
                    ))
                })?,
            size_function: unresolved
                .size_function
                .ok_or_else(|| DemesError::EpochError("end_time unresolved".to_string()))?,
            cloning_rate: unresolved
                .cloning_rate
                .ok_or_else(|| DemesError::EpochError("end_time unresolved".to_string()))?
                .try_into()
                .map_err(|_| {
                    DemesError::EpochError(format!(
                        "invalid cloning_rate: {:?}",
                        unresolved.cloning_rate
                    ))
                })?,

            selfing_rate: unresolved
                .selfing_rate
                .ok_or_else(|| DemesError::EpochError("end_time unresolved".to_string()))?
                .try_into()?,
        })
    }

    fn resolved_time_to_generations(
        &mut self,
        generation_time: GenerationTime,
        rounding: fn(Time, GenerationTime) -> Time,
    ) -> Result<(), DemesError> {
        self.start_time = match convert_resolved_time_to_generations(
            generation_time,
            rounding,
            DemesError::EpochError,
            "start_time is unresolved",
            Some(self.start_time),
        ) {
            Ok(time) => time,
            Err(e) => return Err(e),
        };
        self.end_time = match convert_resolved_time_to_generations(
            generation_time,
            rounding,
            DemesError::EpochError,
            "end_time is unresolved",
            Some(self.end_time),
        ) {
            Ok(time) => time,
            Err(e) => return Err(e),
        };
        Ok(())
    }

    /// The resolved size function
    pub fn size_function(&self) -> SizeFunction {
        self.size_function
    }

    /// The resolved selfing rate
    pub fn selfing_rate(&self) -> SelfingRate {
        self.selfing_rate
    }

    /// The resolved cloning rate
    pub fn cloning_rate(&self) -> CloningRate {
        self.cloning_rate
    }

    /// The resolved start time
    pub fn start_time(&self) -> Time {
        self.start_time
    }

    /// The resolved end time
    pub fn end_time(&self) -> Time {
        self.end_time
    }

    /// The resolved start size
    pub fn start_size(&self) -> DemeSize {
        self.start_size
    }

    /// The resolved end size
    pub fn end_size(&self) -> DemeSize {
        self.end_size
    }

    /// The resolved time interval
    pub fn time_interval(&self) -> TimeInterval {
        TimeInterval::new(self.start_time(), self.end_time())
    }

    /// Size of Epoch at a given time
    ///
    /// # Errors
    ///
    /// * If `time` is not `end_time < time <= start_time`.
    /// * If `time` fails to convert into [`Time`].
    /// * If conversion from [`f64`] to [`DemeSize`] failes.
    pub fn size_at<F: Into<f64>>(&self, time: F) -> Result<DemeSize, DemesError> {
        let time: f64 = time.into();
        Time::try_from(time)
            .map_err(|_| DemesError::EpochError(format!("invalid time value: {time:?}")))?;

        if time == f64::INFINITY && self.start_time == f64::INFINITY {
            return Ok(self.start_size);
        };

        let start_time = f64::from(self.start_time);
        let end_time = f64::from(self.end_time);
        if time < end_time || time >= start_time {
            return Err(DemesError::EpochError(
                "time is not contained in epoch time span".to_string(),
            ));
        }

        let time_span = start_time - end_time;
        let dt = start_time - time;
        let size = match self.size_function() {
            SizeFunction::Constant => return Ok(self.end_size),
            SizeFunction::Linear => {
                f64::from(self.start_size)
                    + dt * (f64::from(self.end_size) - f64::from(self.start_size)) / time_span
            }
            SizeFunction::Exponential => {
                let r = (f64::from(self.end_size) / f64::from(self.start_size)).ln() / time_span;
                f64::from(self.start_size) * (r * dt).exp()
            }
        };
        let size = DemeSize::try_from(size).map_err(|_| {
            DemesError::EpochError(format!("size calculation led to invalid size: {size}"))
        })?;
        Ok(size)
    }
}

impl UnresolvedEpoch {
    fn resolve_size_function(
        &mut self,
        defaults: &GraphDefaults,
        deme_defaults: &DemeDefaults,
    ) -> Option<()> {
        if self.size_function.is_some() {
            return Some(());
        }

        if self.start_size? == self.end_size? {
            self.size_function = Some(SizeFunction::Constant);
        } else {
            self.size_function =
                defaults.apply_epoch_size_function_defaults(self.size_function, deme_defaults);
        }

        Some(())
    }

    fn resolve_selfing_rate(&mut self, defaults: &GraphDefaults, deme_defaults: &DemeDefaults) {
        if self.selfing_rate.is_none() {
            self.selfing_rate = match deme_defaults.epoch.selfing_rate {
                Some(selfing_rate) => Some(selfing_rate),
                None => match defaults.epoch.selfing_rate {
                    Some(selfing_rate) => Some(selfing_rate),
                    None => Some(InputSelfingRate::default()),
                },
            }
        }
    }

    fn resolve_cloning_rate(&mut self, defaults: &GraphDefaults, deme_defaults: &DemeDefaults) {
        if self.cloning_rate.is_none() {
            self.cloning_rate = match deme_defaults.epoch.cloning_rate {
                Some(cloning_rate) => Some(cloning_rate),
                None => match defaults.epoch.cloning_rate {
                    Some(cloning_rate) => Some(cloning_rate),
                    None => Some(InputCloningRate::default()),
                },
            }
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
            .ok_or_else(|| DemesError::EpochError("failed to resolve size_function".to_string()))
    }

    fn validate_end_time(&self, index: usize, deme_name: &str) -> Result<(), DemesError> {
        match self.end_time {
            Some(time) => time.err_if_not_valid_epoch_end_time(),
            None => Err(DemesError::EpochError(format!(
                "deme {deme_name}, epoch {index}: end time is None",
            ))),
        }
    }

    fn validate_cloning_rate(&self, index: usize, deme_name: &str) -> Result<(), DemesError> {
        match self.cloning_rate {
            Some(value) => {
                if CloningRate::try_from(value).is_err() {
                    Err(DemesError::EpochError(format!(
                        "deme {deme_name}, epoch {index}: invalid cloning_rate: {value:?}"
                    )))
                } else {
                    Ok(())
                }
            }
            None => Err(DemesError::EpochError(format!(
                "deme {deme_name}, epoch {index}:cloning_rate is None",
            ))),
        }
    }

    fn validate_selfing_rate(&self, index: usize, deme_name: &str) -> Result<(), DemesError> {
        match self.selfing_rate {
            Some(value) => SelfingRate::try_from(value)
                .map_err(|_| DemesError::EpochError(format!("invalid selfing_rate: {value:?}")))
                .map(|_| ()),
            None => Err(DemesError::EpochError(format!(
                "deme {deme_name}, epoch {index}: selfing_rate is None",
            ))),
        }
    }

    fn validate_size_function(
        &self,
        index: usize,
        deme_name: &str,
        start_size: InputDemeSize,
        end_size: InputDemeSize,
    ) -> Result<(), DemesError> {
        let size_function = self.size_function.ok_or_else(|| {
            DemesError::EpochError(format!(
                "deme {deme_name}, epoch {index}:size function is None",
            ))
        })?;

        let is_constant = matches!(size_function, SizeFunction::Constant);

        if (is_constant && start_size != end_size) || (!is_constant && start_size == end_size) {
            Err(DemesError::EpochError(format!(
                "deme {}, index{}: start_size ({:?}) == end_size ({:?}) paired with invalid size_function: {}",
                deme_name, index, self.start_size, self.end_size, size_function
            )))
        } else {
            Ok(())
        }
    }

    fn validate(&self, index: usize, deme_name: &str) -> Result<(), DemesError> {
        let start_size = self.start_size.ok_or_else(|| {
            DemesError::EpochError(format!(
                "deme {deme_name}, epoch {index}: start_size is None",
            ))
        })?;
        DemeSize::try_from(start_size)
            .map_err(|_| DemesError::EpochError(format!("invalid start_size: {start_size:?}")))?;
        let end_size = self.end_size.ok_or_else(|| {
            DemesError::EpochError(format!("deme {deme_name}, epoch {index}: end_size is None",))
        })?;
        DemeSize::try_from(end_size)
            .map_err(|_| DemesError::EpochError(format!("invalid end_size: {end_size:?}")))?;
        self.validate_end_time(index, deme_name)?;
        self.validate_cloning_rate(index, deme_name)?;
        self.validate_selfing_rate(index, deme_name)?;
        self.validate_size_function(index, deme_name, start_size, end_size)
    }
}

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedDeme {
    name: String,
    #[serde(default = "String::default")]
    description: String,
    #[serde(skip)]
    ancestor_map: DemeMap,
    #[serde(skip)]
    ancestor_indexes: Vec<usize>,
    #[serde(default = "Vec::<UnresolvedEpoch>::default")]
    epochs: Vec<UnresolvedEpoch>,
    #[serde(flatten)]
    history: UnresolvedDemeHistory,
}

/// A resolved deme.
#[derive(Clone, Debug, Serialize)]
pub struct Deme {
    name: String,
    description: String,
    #[serde(skip)]
    ancestor_map: DemeMap,
    #[serde(skip)]
    ancestor_indexes: Vec<usize>,
    epochs: Vec<Epoch>,
    ancestors: Vec<String>,
    proportions: Vec<Proportion>,
    start_time: Time,
}

impl Deme {
    fn resolved_time_to_generations(
        &mut self,
        generation_time: GenerationTime,
        rounding: fn(Time, GenerationTime) -> Time,
    ) -> Result<(), DemesError> {
        self.start_time = match convert_resolved_time_to_generations(
            generation_time,
            rounding,
            DemesError::DemeError,
            &format!("start_time unresolved for deme: {}", self.name),
            Some(self.start_time),
        ) {
            Ok(time) => time,
            Err(e) => return Err(e),
        };
        self.epochs
            .iter_mut()
            .try_for_each(|epoch| epoch.resolved_time_to_generations(generation_time, rounding))?;

        let valid = |w: (Time, Time)| {
            if w.1 >= w.0 {
                Err(DemesError::EpochError(
                    "conversion to generations resulted in an invalid Epoch".to_string(),
                ))
            } else {
                Ok(())
            }
        };

        self.start_times()
            .zip(self.end_times())
            .try_for_each(valid)?;

        self.end_times()
            .take(self.num_epochs() - 1)
            .zip(self.end_times().skip(1))
            .try_for_each(valid)?;

        Ok(())
    }

    /// Iterator over resolved epoch start times
    pub fn start_times(&self) -> impl Iterator<Item = Time> + '_ {
        DemeEpochStartTimesIterator::new(self)
    }

    /// The resolved start time
    pub fn start_time(&self) -> Time {
        self.start_time
    }

    /// Deme name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The resolved time interval
    pub fn time_interval(&self) -> TimeInterval {
        TimeInterval::new(self.start_time(), self.end_time())
    }

    /// Number of ancestors
    pub fn num_ancestors(&self) -> usize {
        self.ancestors.len()
    }

    /// Iterator over resolved epoch start sizes.
    pub fn start_sizes(&self) -> impl Iterator<Item = DemeSize> + '_ {
        DemeEpochStartSizesIterator::new(self)
    }

    /// Iterator over resolved epoch end sizes
    pub fn end_sizes(&self) -> impl Iterator<Item = DemeSize> + '_ {
        DemeEpochEndSizesIterator::new(self)
    }

    /// Itertor over resolved epoch end times
    pub fn end_times(&self) -> impl Iterator<Item = Time> + '_ {
        DemeEpochEndTimesIterator::new(self)
    }

    /// End time of the deme.
    ///
    /// Obtained from the value stored in the most
    /// recent epoch.
    pub fn end_time(&self) -> Time {
        assert!(!self.epochs.is_empty());
        self.epochs[self.epochs.len() - 1].end_time()
    }

    /// Hash map of ancestor name to ancestor deme
    #[deprecated(note = "Use Deme::ancestor_names and/or Deme::ancestor_indexes instead")]
    pub fn ancestors(&self) -> &DemeMap {
        &self.ancestor_map
    }

    /// Resolved start size
    pub fn start_size(&self) -> DemeSize {
        self.epochs[0].start_size()
    }

    /// Resolved end size
    pub fn end_size(&self) -> DemeSize {
        assert!(!self.epochs.is_empty());
        self.epochs[self.epochs.len() - 1].end_size()
    }

    /// Names of ancestor demes.
    ///
    /// Empty if no ancestors.
    pub fn ancestor_names(&self) -> &[String] {
        &self.ancestors
    }

    /// Indexes of ancestor demes.
    ///
    /// Empty if no ancestors.
    pub fn ancestor_indexes(&self) -> &[usize] {
        debug_assert_eq!(self.ancestor_indexes.len(), self.ancestors.len());
        &self.ancestor_indexes
    }

    /// Description string
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Obtain the number of [`Epoch`](crate::Epoch) instances.
    ///
    /// # Examples
    ///
    /// See [`here`](crate::SizeFunction).
    pub fn num_epochs(&self) -> usize {
        self.epochs.len()
    }

    /// Resolved epochs
    pub fn epochs(&self) -> &[Epoch] {
        &self.epochs
    }

    /// Returns a copy of the [`Epoch`](crate::Epoch) at index `epoch`.
    ///
    /// # Examples
    ///
    /// See [`here`](crate::SizeFunction) for examples.
    pub fn get_epoch(&self, epoch: usize) -> Option<Epoch> {
        self.epochs.get(epoch).copied()
    }

    /// Resolved proportions
    pub fn proportions(&self) -> &[Proportion] {
        &self.proportions
    }

    /// Size of Deme at a given time
    ///
    /// # Errors
    ///
    /// See [`Epoch::size_at`] for details.
    pub fn size_at<F: Into<f64>>(&self, time: F) -> Result<Option<DemeSize>, DemesError> {
        let time: f64 = time.into();
        Time::try_from(time)
            .map_err(|_| DemesError::DemeError(format!("invalid time: {time:?}")))?;

        if time == f64::INFINITY && self.start_time == f64::INFINITY {
            return Ok(Some(self.epochs()[0].start_size));
        };

        let epoch = self.epochs().iter().find(|x| {
            x.time_interval()
                .contains_exclusive_start_inclusive_end(time)
        });

        match epoch {
            None => Ok(None),
            Some(e) => Ok(Some(e.size_at(time)?)),
        }
    }
}

impl TryFrom<UnresolvedDeme> for Deme {
    type Error = DemesError;

    fn try_from(value: UnresolvedDeme) -> Result<Self, Self::Error> {
        let mut epochs = vec![];
        let start_time = value.history.start_time.ok_or_else(|| {
            DemesError::DemeError(format!("deme {} start_time is not resolved", value.name))
        })?;
        let mut epoch_start_time = start_time;
        for hdm_epoch in value.epochs.into_iter() {
            let end_time = hdm_epoch
                .end_time
                .ok_or_else(|| DemesError::EpochError("epoch end time unresolved".to_string()))?;
            let e = Epoch::new_from_unresolved(epoch_start_time, hdm_epoch)?;
            epoch_start_time = end_time;
            epochs.push(e);
        }
        let input_proportions = value.history.proportions.ok_or_else(|| {
            DemesError::PulseError("pulse proportions are unresolved".to_string())
        })?;
        let mut proportions = vec![];
        for p in input_proportions {
            proportions.push(Proportion::try_from(p)?);
        }
        Ok(Self {
            description: value.description,
            ancestor_map: value.ancestor_map,
            ancestor_indexes: value.ancestor_indexes,
            epochs,
            ancestors: value.history.ancestors.ok_or_else(|| {
                DemesError::DemeError(format!("deme {} ancestors are not resolved", value.name))
            })?,
            proportions,
            start_time: start_time.try_into().map_err(|_| {
                DemesError::DemeError(format!("invalid start_time: {start_time:?}"))
            })?,
            name: value.name,
        })
    }
}

impl PartialEq for Deme {
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

macro_rules! build_epoch_data_iterator {
    ($name: ident, $item: ty, $function: ident) => {
        struct $name<'deme> {
            deme: &'deme Deme,
            epoch: usize,
        }

        impl<'deme> $name<'deme> {
            fn new(deme: &'deme Deme) -> Self {
                Self { deme, epoch: 0 }
            }
        }

        impl<'deme> Iterator for $name<'deme> {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                match self.deme.epochs.get(self.epoch) {
                    Some(epoch) => {
                        self.epoch += 1;
                        Some(epoch.$function())
                    }
                    None => None,
                }
            }
        }
    };
}

build_epoch_data_iterator!(DemeEpochStartTimesIterator, Time, start_time);
build_epoch_data_iterator!(DemeEpochEndTimesIterator, Time, end_time);
build_epoch_data_iterator!(DemeEpochStartSizesIterator, DemeSize, start_size);
build_epoch_data_iterator!(DemeEpochEndSizesIterator, DemeSize, end_size);

struct DOIIterator<'graph> {
    graph: &'graph Graph,
    index: usize,
}

impl<'graph> DOIIterator<'graph> {
    fn new(graph: &'graph Graph) -> Self {
        Self { graph, index: 0 }
    }
}

impl<'graph> Iterator for DOIIterator<'graph> {
    type Item = &'graph str;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.graph.doi {
            Some(doi) => {
                let temp = self.index;
                self.index += 1;
                //doi.get(temp)
                match doi.get(temp) {
                    Some(v) => Some(v.as_str()),
                    None => None,
                }
            }
            None => None,
        }
    }
}

/// HDM data for a [`Deme`](crate::Deme)
#[derive(Default, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedDemeHistory {
    #[allow(missing_docs)]
    // NOTE: we use option here because
    // an empty vector in the input means
    // "no ancestors" (i.e., the demes themselves are
    // the most ancient).
    // When there are toplevel deme defaults,
    // we only fill them in when this value is None
    pub ancestors: Option<Vec<String>>,
    #[allow(missing_docs)]
    pub proportions: Option<Vec<InputProportion>>,
    #[allow(missing_docs)]
    pub start_time: Option<InputTime>,
    #[serde(default = "DemeDefaults::default")]
    #[serde(skip_serializing)]
    #[allow(missing_docs)]
    pub defaults: DemeDefaults,
}

impl PartialEq for UnresolvedDeme {
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

impl Eq for UnresolvedDeme {}

impl UnresolvedDeme {
    pub(crate) fn new_via_builder(
        name: &str,
        epochs: Vec<UnresolvedEpoch>,
        history: UnresolvedDemeHistory,
        description: Option<&str>,
    ) -> Self {
        let description = match description {
            Some(desc) => desc.to_string(),
            None => String::default(),
        };
        Self {
            name: name.to_string(),
            epochs,
            history,
            description,
            ..Default::default()
        }
    }

    fn resolve_times(
        &mut self,
        deme_map: &DemeMap,
        demes: &[UnresolvedDeme],
        defaults: &GraphDefaults,
    ) -> Result<(), DemesError> {
        // apply top-level default if it exists

        self.history.start_time = match self.history.start_time {
            Some(start_time) => Some(start_time),
            None => match defaults.deme.start_time {
                Some(start_time) => Some(start_time),
                None => Some(InputTime::default_deme_start_time()),
            },
        };

        if self
            .history
            .ancestors
            .as_ref()
            .ok_or_else(|| DemesError::DemeError("unexpected None for deme ancestors".to_string()))?
            .is_empty()
            && self.start_time_resolved_or(|| {
                DemesError::DemeError(format!("deme {}: start_time unresolved", self.name))
            })? != InputTime::default_deme_start_time()
        {
            return Err(DemesError::DemeError(format!(
                "deme {} has finite start time but no ancestors",
                self.name
            )));
        }

        if self.get_num_ancestors()? == 1 {
            let first_ancestor_name = &self.get_ancestor_names()?[0];

            let deme_start_time = match self.history.start_time {
                Some(start_time) => {
                    if start_time == InputTime::default_deme_start_time() {
                        let first_ancestor_deme = get_deme!(first_ancestor_name, deme_map, demes)
                            .ok_or_else(|| {
                            DemesError::DemeError(
                                "fatal error: ancestor maps to no Deme object".to_string(),
                            )
                        })?;
                        first_ancestor_deme.get_end_time()?.into()
                    } else {
                        start_time
                    }
                }
                None => InputTime::default_deme_start_time(),
            };

            deme_start_time.err_if_not_valid_deme_start_time()?;
            self.history.start_time = Some(deme_start_time);
        }

        for ancestor in self.get_ancestor_names()?.iter() {
            let a = get_deme!(ancestor, deme_map, demes).ok_or_else(|| {
                DemesError::DemeError(format!(
                    "ancestor {ancestor} not present in global deme map",
                ))
            })?;
            let t = a.get_time_interval()?;
            if !t.contains_start_time(self.get_start_time()?) {
                return Err(DemesError::DemeError(format!(
                    "Ancestor {} does not exist at deme {}'s start_time",
                    ancestor, self.name
                )));
            }
        }

        // last epoch end time defaults to 0,
        // unless defaults are specified
        let last_epoch_ref = self
            .epochs
            .last_mut()
            .ok_or_else(|| DemesError::DemeError("epochs are empty".to_string()))?;
        if last_epoch_ref.end_time.is_none() {
            last_epoch_ref.end_time = match self.history.defaults.epoch.end_time {
                Some(end_time) => Some(end_time),
                None => match defaults.epoch.end_time {
                    Some(end_time) => Some(end_time),
                    None => Some(InputTime::default_epoch_end_time()),
                },
            }
        }

        // apply default epoch start times
        for epoch in self.epochs.iter_mut() {
            match epoch.end_time {
                Some(end_time) => {
                    Time::try_from(end_time).map_err(|_| {
                        DemesError::EpochError(format!("invalid end_time: {end_time:?}"))
                    })?;
                }
                None => {
                    epoch.end_time = match self.history.defaults.epoch.end_time {
                        Some(end_time) => Some(end_time),
                        None => defaults.epoch.end_time,
                    }
                }
            }
        }

        let mut last_time = f64::from(self.get_start_time()?);
        for (i, epoch) in self.epochs.iter().enumerate() {
            let end_time = f64::from(epoch.end_time.ok_or_else(|| {
                DemesError::EpochError(format!(
                    "deme: {}, epoch: {i} end time must be specified",
                    self.name
                ))
            })?);

            if end_time >= last_time {
                return Err(DemesError::EpochError(
                    "Epoch end times must be listed in decreasing order".to_string(),
                ));
            }
            last_time = end_time;
            Time::try_from(
                epoch
                    .end_time
                    .ok_or_else(|| DemesError::EpochError("end_time is None".to_string()))?,
            )
            .map_err(|_| {
                DemesError::EpochError(format!("invalid end_time: {:?}", epoch.end_time))
            })?;
        }

        Ok(())
    }

    fn resolve_first_epoch_sizes(
        &mut self,
        defaults: &GraphDefaults,
    ) -> Result<Option<InputDemeSize>, DemesError> {
        let self_defaults = self.history.defaults.clone();
        let epoch_sizes = {
            let temp_epoch = self.epochs.get_mut(0).ok_or_else(|| {
                DemesError::DemeError(format!("deme {} has no epochs", self.name))
            })?;

            temp_epoch.start_size = match temp_epoch.start_size {
                Some(start_size) => Some(start_size),
                None => self_defaults.epoch.start_size,
            };
            temp_epoch.end_size = match temp_epoch.end_size {
                Some(end_size) => Some(end_size),
                None => self_defaults.epoch.end_size,
            };

            defaults.apply_epoch_size_defaults(temp_epoch);
            if temp_epoch.start_size.is_none() && temp_epoch.end_size.is_none() {
                return Err(DemesError::EpochError(format!(
                    "first epoch of deme {} must define one or both of start_size and end_size",
                    self.name
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

        let epoch_start_size = epoch_sizes.0.ok_or_else(|| {
            DemesError::EpochError(format!(
                "first epoch of {} has unresolved start_size",
                self.name
            ))
        })?;
        let epoch_end_size = epoch_sizes.1.ok_or_else(|| {
            DemesError::EpochError(format!(
                "first epoch of {} has unresolved end_size",
                self.name
            ))
        })?;

        let start_time = self.history.start_time.ok_or_else(|| {
            DemesError::EpochError(format!("deme {} start_time is None", self.name))
        })?;

        if start_time == InputTime::default_deme_start_time() && epoch_sizes.0 != epoch_sizes.1 {
            let msg = format!(
                    "first epoch of deme {} cannot have varying size and an infinite time interval: start_size = {}, end_size = {}",
                    self.name, f64::from(epoch_start_size), f64::from(epoch_end_size),
                );
            return Err(DemesError::EpochError(msg));
        }

        Ok(Some(epoch_end_size))
    }

    fn resolve_sizes(&mut self, defaults: &GraphDefaults) -> Result<(), DemesError> {
        let mut last_end_size = self.resolve_first_epoch_sizes(defaults)?;
        let local_defaults = self.history.defaults.clone();
        for epoch in self.epochs.iter_mut().skip(1) {
            match epoch.start_size {
                Some(_) => (),
                None => match local_defaults.epoch.start_size {
                    Some(start_size) => epoch.start_size = Some(start_size),
                    None => match defaults.epoch.start_size {
                        Some(start_size) => epoch.start_size = Some(start_size),
                        None => epoch.start_size = last_end_size,
                    },
                },
            }
            match epoch.end_size {
                Some(_) => (),
                None => match local_defaults.epoch.end_size {
                    Some(end_size) => epoch.end_size = Some(end_size),
                    None => match defaults.epoch.end_size {
                        Some(end_size) => epoch.end_size = Some(end_size),
                        None => epoch.end_size = epoch.start_size,
                    },
                },
            }
            last_end_size = epoch.end_size;
        }
        Ok(())
    }

    fn resolve_proportions(&mut self) -> Result<(), DemesError> {
        let num_ancestors = self.get_num_ancestors()?;

        let proportions = self
            .history
            .proportions
            .as_mut()
            .ok_or_else(|| DemesError::DemeError("proportions is None".to_string()))?;

        if proportions.is_empty() && num_ancestors == 1 {
            proportions.push(InputProportion::from(1.0));
        }

        if num_ancestors != proportions.len() {
            return Err(DemesError::DemeError(format!(
                "deme {} ancestors and proportions have different lengths",
                self.name
            )));
        }
        Ok(())
    }

    fn check_empty_epochs(&mut self) {
        if self.epochs.is_empty() {
            self.epochs.push(UnresolvedEpoch::default());
        }
    }

    fn apply_toplevel_defaults(&mut self, defaults: &GraphDefaults) {
        if self.history.ancestors.is_none() {
            self.history.ancestors = match &defaults.deme.ancestors {
                Some(ancestors) => Some(ancestors.to_vec()),
                None => Some(vec![]),
            }
        }

        if self.history.proportions.is_none() {
            self.history.proportions = match &defaults.deme.proportions {
                Some(proportions) => Some(proportions.to_vec()),
                None => Some(vec![]),
            }
        }
    }

    fn validate_ancestor_uniqueness(&self, deme_map: &DemeMap) -> Result<(), DemesError> {
        match &self.history.ancestors {
            Some(ancestors) => {
                let mut ancestor_set = HashSet::<String>::default();
                for ancestor in ancestors {
                    if ancestor == &self.name {
                        return Err(DemesError::DemeError(format!(
                            "deme: {} lists itself as an ancestor",
                            self.name
                        )));
                    }
                    if !deme_map.contains_key(ancestor) {
                        return Err(DemesError::DemeError(format!(
                            "deme: {} lists invalid ancestor: {ancestor}",
                            self.name
                        )));
                    }
                    if ancestor_set.contains(ancestor) {
                        return Err(DemesError::DemeError(format!(
                            "deme: {} lists ancestor: {ancestor} multiple times",
                            self.name
                        )));
                    }
                    ancestor_set.insert(ancestor.clone());
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    // Make the internal data match the MDM spec
    fn resolve(
        &mut self,
        deme_map: &DemeMap,
        demes: &[UnresolvedDeme],
        defaults: &GraphDefaults,
    ) -> Result<(), DemesError> {
        self.history.defaults.validate()?;
        self.apply_toplevel_defaults(defaults);
        self.validate_ancestor_uniqueness(deme_map)?;
        self.check_empty_epochs();
        assert!(self.ancestor_map.is_empty());
        self.resolve_times(deme_map, demes, defaults)?;
        self.resolve_sizes(defaults)?;
        let self_defaults = self.history.defaults.clone();
        self.epochs
            .iter_mut()
            .try_for_each(|e| e.resolve(defaults, &self_defaults))?;
        self.resolve_proportions()?;

        let mut ancestor_map = DemeMap::default();
        let ancestors = self.history.ancestors.as_ref().ok_or_else(|| {
            DemesError::DemeError(format!("deme {}: ancestors are None", self.name))
        })?;
        for ancestor in ancestors {
            let deme = deme_map.get(ancestor).ok_or_else(|| {
                DemesError::DemeError(format!("invalid ancestor of {}: {ancestor}", self.name))
            })?;
            ancestor_map.insert(ancestor.clone(), *deme);
            self.ancestor_indexes.push(*deme);
        }
        self.ancestor_map = ancestor_map;
        Ok(())
    }

    fn validate_start_time(&self) -> Result<(), DemesError> {
        match self.history.start_time {
            Some(start_time) => {
                Time::try_from(start_time).map_err(|_| {
                    DemesError::DemeError(format!("invalid start_time: {start_time:?}"))
                })?;
                start_time.err_if_not_valid_deme_start_time()
            }
            None => Err(DemesError::DemeError("start_time is None".to_string())),
        }
    }

    fn start_time_resolved_or<F: FnOnce() -> DemesError>(
        &self,
        err: F,
    ) -> Result<InputTime, DemesError> {
        self.history.start_time.ok_or_else(err)
    }

    // Names must be valid Python identifiers
    // https://docs.python.org/3/reference/lexical_analysis.html#identifiers
    pub(crate) fn validate_name(&self) -> Result<(), DemesError> {
        let python_identifier = match regex::Regex::new(r"^[^\d\W]\w*$") {
            Ok(p) => p,
            Err(_) => {
                return Err(DemesError::DemeError(
                    "failed to biuld python_identifier regex".to_string(),
                ))
            }
        };
        if python_identifier.is_match(&self.name) {
            Ok(())
        } else {
            Err(DemesError::DemeError(format!(
                "invalid deme name: {}:",
                self.name
            )))
        }
    }

    fn validate(&self) -> Result<(), DemesError> {
        self.validate_name()?;
        self.validate_start_time()?;
        if self.epochs.is_empty() {
            return Err(DemesError::DemeError(format!(
                "no epochs for deme {}",
                self.name
            )));
        }

        self.epochs
            .iter()
            .enumerate()
            .try_for_each(|(i, e)| e.validate(i, &self.name))?;

        let proportions = self
            .history
            .proportions
            .as_ref()
            .ok_or_else(|| DemesError::DemeError("proportions is None".to_string()))?;
        for p in proportions.iter() {
            Proportion::try_from(*p)?;
        }

        if !proportions.is_empty() {
            let sum_proportions: f64 = proportions.iter().map(|p| f64::from(*p)).sum();
            // NOTE: this is same default as Python's math.isclose().
            if (sum_proportions - 1.0).abs() > 1e-9 {
                return Err(DemesError::DemeError(format!(
                    "proportions for deme {} should sum to ~1.0, got: {sum_proportions}",
                    self.name
                )));
            }
        }

        Ok(())
    }

    fn get_time_interval(&self) -> Result<TimeInterval, DemesError> {
        let start_time = self.get_start_time()?;
        let end_time = self.get_end_time()?;
        Ok(TimeInterval::new(start_time, end_time))
    }

    fn get_ancestor_names(&self) -> Result<&[String], DemesError> {
        match &self.history.ancestors {
            Some(ancestors) => Ok(ancestors),
            None => Err(DemesError::DemeError(format!(
                "deme {} ancestors are unresolved",
                self.name
            ))),
        }
    }

    fn get_start_time(&self) -> Result<Time, DemesError> {
        match self.history.start_time.ok_or_else(|| {
            DemesError::DemeError(format!("deme {} start_time is unresolved", self.name))
        }) {
            Ok(value) => value.try_into(),
            Err(e) => Err(e),
        }
    }

    fn get_end_time(&self) -> Result<Time, DemesError> {
        match self
            .epochs
            .last()
            .as_ref()
            .ok_or_else(|| DemesError::DemeError(format!("deme {} has no epochs", self.name)))?
            .end_time
            .ok_or_else(|| {
                DemesError::DemeError(format!(
                    "last epoch of deme {} end_time unresolved",
                    self.name
                ))
            }) {
            Ok(value) => value.try_into(),
            Err(e) => Err(e),
        }
    }

    fn get_num_ancestors(&self) -> Result<usize, DemesError> {
        Ok(self
            .history
            .ancestors
            .as_ref()
            .ok_or_else(|| {
                DemesError::DemeError(format!("deme {} ancestors are unresolved", self.name))
            })?
            .len())
    }
}

type DemeMap = HashMap<String, usize>;

fn deme_name_exists<F: FnOnce(String) -> DemesError>(
    map: &DemeMap,
    name: &str,
    err: F,
) -> Result<(), DemesError> {
    if !map.contains_key(name) {
        Err(err(format!("deme {name} does not exist")))
    } else {
        Ok(())
    }
}

#[derive(Default, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphDefaultInput {
    #[serde(flatten)]
    defaults: GraphDefaults,
}

/// Top-level defaults
#[derive(Default, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphDefaults {
    #[allow(missing_docs)]
    #[serde(default = "UnresolvedEpoch::default")]
    #[allow(missing_docs)]
    pub epoch: UnresolvedEpoch,
    #[serde(default = "UnresolvedMigration::default")]
    #[allow(missing_docs)]
    pub migration: UnresolvedMigration,
    #[serde(default = "UnresolvedPulse::default")]
    #[allow(missing_docs)]
    pub pulse: UnresolvedPulse,
    #[serde(default = "TopLevelDemeDefaults::default")]
    #[allow(missing_docs)]
    pub deme: TopLevelDemeDefaults,
}

impl GraphDefaults {
    // This fn exists so that we catch invalid inputs
    // prior to resolution.  During resolution,
    // we only visit the top-level defaults if needed.
    // Thus, we will miss invalid inputs if we wait
    // until resolution.
    fn validate(&self) -> Result<(), DemesError> {
        self.epoch.validate_as_default()?;
        self.pulse.validate_as_default()?;
        self.migration.validate()?;
        self.deme.validate()
    }

    fn apply_default_epoch_start_size(
        &self,
        start_size: Option<InputDemeSize>,
    ) -> Option<InputDemeSize> {
        if start_size.is_some() {
            return start_size;
        }
        self.epoch.start_size
    }

    fn apply_default_epoch_end_size(
        &self,
        end_size: Option<InputDemeSize>,
    ) -> Option<InputDemeSize> {
        if end_size.is_some() {
            return end_size;
        }
        self.epoch.end_size
    }

    fn apply_epoch_size_defaults(&self, epoch: &mut UnresolvedEpoch) {
        epoch.start_size = self.apply_default_epoch_start_size(epoch.start_size);
        epoch.end_size = self.apply_default_epoch_end_size(epoch.end_size);
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

    fn apply_pulse_defaults(&self, other: &mut UnresolvedPulse) {
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

/// Top-level defaults for a [`Deme`](crate::Deme).
///
/// This type is used as a member of
/// [`GraphDefaults`](crate::GraphDefaults)
#[derive(Clone, Default, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopLevelDemeDefaults {
    #[allow(missing_docs)]
    pub description: Option<String>,
    #[allow(missing_docs)]
    pub start_time: Option<InputTime>,
    #[allow(missing_docs)]
    pub ancestors: Option<Vec<String>>,
    #[allow(missing_docs)]
    pub proportions: Option<Vec<InputProportion>>,
}

impl TopLevelDemeDefaults {
    fn validate(&self) -> Result<(), DemesError> {
        if let Some(value) = self.start_time {
            Time::try_from(value)
                .map_err(|_| DemesError::DemeError(format!("invalid start_time: {value:?}")))?;
        }

        match &self.proportions {
            Some(value) => {
                for v in value {
                    if Proportion::try_from(*v).is_err() {
                        return Err(DemesError::GraphError(format!(
                            "invalid default proportion: {v:?}"
                        )));
                    }
                }
            }
            None => (),
        }

        Ok(())
    }
}

/// Deme-level defaults
#[derive(Clone, Default, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemeDefaults {
    #[allow(missing_docs)]
    pub epoch: UnresolvedEpoch,
}

impl DemeDefaults {
    fn validate(&self) -> Result<(), DemesError> {
        self.epoch.validate_as_default()
    }
}

/// Top-level metadata
///
/// # Examples
///
/// ```
/// #[derive(serde::Deserialize)]
/// struct MyMetaData {
///    foo: i32,
///    bar: String
/// }
///
/// let yaml = "
/// time_units: generations
/// metadata:
///  foo: 1
///  bar: bananas
/// demes:
///  - name: A
///    epochs:
///     - start_size: 100
/// ";
///
/// let graph = demes::loads(yaml).unwrap();
/// let yaml_metadata = graph.metadata().unwrap().as_yaml_string().unwrap();
/// let my_metadata: MyMetaData = serde_yaml::from_str(&yaml_metadata).unwrap();
/// assert_eq!(my_metadata.foo, 1);
/// assert_eq!(&my_metadata.bar, "bananas");
/// ```
#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Metadata {
    #[serde(flatten, deserialize_with = "require_non_empty_metadata")]
    metadata: std::collections::BTreeMap<String, serde_yaml::Value>,
}

fn require_non_empty_metadata<'de, D>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, serde_yaml::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let buf = std::collections::BTreeMap::<String, serde_yaml::Value>::deserialize(deserializer)?;

    if !buf.is_empty() {
        Ok(buf)
    } else {
        Err(serde::de::Error::custom(
            "metadata: cannot be an empty mapping".to_string(),
        ))
    }
}

impl Metadata {
    /// `true` if metadata is present, `false` otherwise
    fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    /// Return the metadata as YAML
    pub fn as_yaml_string(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self.metadata)
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedGraph {
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
    generation_time: Option<InputGenerationTime>,
    pub(crate) demes: Vec<UnresolvedDeme>,
    #[serde(default = "Vec::<UnresolvedMigration>::default")]
    #[serde(rename = "migrations")]
    #[serde(skip_serializing)]
    input_migrations: Vec<UnresolvedMigration>,
    #[serde(default = "Vec::<AsymmetricMigration>::default")]
    #[serde(rename = "migrations")]
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Vec::<AsymmetricMigration>::is_empty")]
    resolved_migrations: Vec<AsymmetricMigration>,
    #[serde(default = "Vec::<UnresolvedPulse>::default")]
    pulses: Vec<UnresolvedPulse>,
    #[serde(skip)]
    deme_map: DemeMap,
}

impl UnresolvedGraph {
    pub(crate) fn new(
        time_units: TimeUnits,
        generation_time: Option<InputGenerationTime>,
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
            demes: Vec::<UnresolvedDeme>::default(),
            input_migrations: Vec::<UnresolvedMigration>::default(),
            resolved_migrations: Vec::<AsymmetricMigration>::default(),
            pulses: Vec::<UnresolvedPulse>::default(),
            deme_map: DemeMap::default(),
        }
    }

    pub(crate) fn add_deme(&mut self, deme: UnresolvedDeme) {
        self.demes.push(deme);
    }

    pub(crate) fn add_migration<I: Into<UnresolvedMigration>>(&mut self, migration: I) {
        self.input_migrations.push(migration.into());
    }

    pub(crate) fn add_pulse(
        &mut self,
        sources: Option<Vec<String>>,
        dest: Option<String>,
        time: Option<InputTime>,
        proportions: Option<Vec<InputProportion>>,
    ) {
        self.pulses.push(UnresolvedPulse {
            sources,
            dest,
            time,
            proportions,
        });
    }

    fn build_deme_map(&self) -> Result<DemeMap, DemesError> {
        let mut rv = DemeMap::default();

        for (i, deme) in self.demes.iter().enumerate() {
            if rv.contains_key(&deme.name) {
                return Err(DemesError::DemeError(format!(
                    "duplicate deme name: {}",
                    deme.name,
                )));
            }
            rv.insert(deme.name.clone(), i);
        }

        Ok(rv)
    }

    fn resolve_asymmetric_migration(
        &mut self,
        source: String,
        dest: String,
        rate: InputMigrationRate,
        start_time: Option<InputTime>,
        end_time: Option<InputTime>,
    ) -> Result<(), DemesError> {
        let source_deme = get_deme!(&source, &self.deme_map, &self.demes).ok_or_else(|| {
            crate::DemesError::MigrationError(format!("invalid source deme name {source}"))
        })?;
        let dest_deme = get_deme!(&dest, &self.deme_map, &self.demes).ok_or_else(|| {
            crate::DemesError::MigrationError(format!("invalid dest deme name {dest}"))
        })?;

        let start_time = match start_time {
            Some(t) => t,
            None => {
                std::cmp::min(source_deme.get_start_time()?, dest_deme.get_start_time()?).into()
            }
        };

        let end_time = match end_time {
            Some(t) => t,
            None => std::cmp::max(source_deme.get_end_time()?, dest_deme.get_end_time()?).into(),
        };

        deme_name_exists(&self.deme_map, &source, DemesError::MigrationError)?;
        deme_name_exists(&self.deme_map, &dest, DemesError::MigrationError)?;

        let a = AsymmetricMigration {
            source,
            dest,
            rate: rate.try_into()?,
            start_time: start_time.try_into().map_err(|_| {
                DemesError::MigrationError(format!("invalid start_time: {start_time:?}"))
            })?,
            end_time: end_time.try_into().map_err(|_| {
                DemesError::MigrationError(format!("invalid end_time: {end_time:?}"))
            })?,
        };

        self.resolved_migrations.push(a);

        Ok(())
    }

    fn process_input_asymmetric_migration(
        &mut self,
        u: &UnresolvedMigration,
    ) -> Result<(), DemesError> {
        self.resolve_asymmetric_migration(
            u.source.clone().ok_or_else(|| {
                DemesError::MigrationError("migration source is None".to_string())
            })?,
            u.dest
                .clone()
                .ok_or_else(|| DemesError::MigrationError("migration dest is None".to_string()))?,
            u.rate
                .ok_or_else(|| DemesError::MigrationError("migration rate is None".to_string()))?,
            u.start_time,
            u.end_time,
        )
    }

    fn process_input_symmetric_migration(
        &mut self,
        u: &UnresolvedMigration,
    ) -> Result<(), DemesError> {
        let demes = u
            .demes
            .as_ref()
            .ok_or_else(|| DemesError::MigrationError("migration demes is None".to_string()))?;

        if demes.len() < 2 {
            return Err(DemesError::MigrationError(
                "the demes field of a migration mut contain at least two demes".to_string(),
            ));
        }

        let rate = u
            .rate
            .ok_or_else(|| DemesError::MigrationError("migration rate is None".to_string()))?;

        // Each input symmetric migration becomes two AsymmetricMigration instances
        for (i, source_name) in demes.iter().enumerate().take(demes.len() - 1) {
            for dest_name in demes.iter().skip(i + 1) {
                if source_name == dest_name {
                    return Err(DemesError::MigrationError(format!(
                        "source/dest demes must differ: {source_name}",
                    )));
                }
                deme_name_exists(&self.deme_map, source_name, DemesError::MigrationError)?;
                deme_name_exists(&self.deme_map, dest_name, DemesError::MigrationError)?;

                let start_time = u.start_time;
                let end_time = u.end_time;

                self.resolve_asymmetric_migration(
                    source_name.to_string(),
                    dest_name.to_string(),
                    rate,
                    start_time,
                    end_time,
                )?;
                self.resolve_asymmetric_migration(
                    dest_name.to_string(),
                    source_name.to_string(),
                    rate,
                    start_time,
                    end_time,
                )?;
            }
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
                Migration::Asymmetric(a) => self.process_input_asymmetric_migration(&a)?,
                Migration::Symmetric(s) => self.process_input_symmetric_migration(&s)?,
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
            unique_times.insert(HashableTime::from(migration.start_time()));
            unique_times.insert(HashableTime::from(migration.end_time()));
        }
        unique_times.retain(|t| f64::from(*t).is_finite());

        let mut end_times = unique_times.into_iter().map(Time::from).collect::<Vec<_>>();

        // REVERSE sort
        end_times.sort_by(|a, b| b.cmp(a));

        let mut start_times = vec![Time::try_from(f64::INFINITY).unwrap()];

        if let Some((_last, elements)) = end_times.split_last() {
            start_times.extend_from_slice(elements);
        }

        start_times
            .into_iter()
            .zip(end_times)
            .map(|times| TimeInterval::new(times.0, times.1))
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
                            let rate = rates[i] + f64::from(migration.rate());
                            if rate > 1.0 + 1e-9 {
                                let msg = format!("migration rate into dest: {} is > 1 in the time interval ({:?}, {:?}]",
                                                  migration.dest(), ti.start_time(), ti.end_time());
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
            let source = get_deme!(&m.source, &self.deme_map, &self.demes).ok_or_else(|| {
                DemesError::MigrationError(format!("invalid migration source: {}", m.source))
            })?;
            let dest = get_deme!(&m.dest, &self.deme_map, &self.demes).ok_or_else(|| {
                DemesError::MigrationError(format!("invalid migration dest: {}", m.dest))
            })?;

            if source.name == dest.name {
                return Err(DemesError::MigrationError(format!(
                    "source: {} == dest: {}",
                    source.name, dest.name
                )));
            }

            {
                let interval = source.get_time_interval()?;
                if !interval.contains_inclusive_start_exclusive_end(m.start_time) {
                    return Err(DemesError::MigrationError(format!(
                            "migration start_time: {:?} does not overlap with existence of source deme {}",
                            m.start_time,
                            source.name
                        )));
                }
                let interval = dest.get_time_interval()?;
                if !interval.contains_inclusive_start_exclusive_end(m.start_time) {
                    return Err(DemesError::MigrationError(format!(
                            "migration start_time: {:?} does not overlap with existence of dest deme {}",
                            m.start_time,
                            dest.name
                        )));
                }
            }

            {
                if !f64::from(m.end_time).is_finite() {
                    return Err(DemesError::MigrationError(format!(
                        "invalid migration end_time: {:?}",
                        m.end_time
                    )));
                }
                let interval = source.get_time_interval()?;
                if !interval.contains_exclusive_start_inclusive_end(m.end_time) {
                    return Err(DemesError::MigrationError(format!(
                            "migration end_time: {:?} does not overlap with existence of source deme {}",
                            m.end_time,
                            source.name
                        )));
                }
                let interval = dest.get_time_interval()?;
                if !interval.contains_exclusive_start_inclusive_end(m.end_time) {
                    return Err(DemesError::MigrationError(format!(
                        "migration end_time: {:?} does not overlap with existence of dest deme {}",
                        m.end_time, dest.name
                    )));
                }
            }

            let interval = m.time_interval();
            if !interval.duration_greater_than_zero() {
                return Err(DemesError::MigrationError(format!(
                    "invalid migration duration: {interval:?} ",
                )));
            }
        }
        self.check_migration_epoch_overlap()?;
        self.validate_input_migration_rates()?;
        Ok(())
    }

    fn resolve_pulses(&mut self) -> Result<(), DemesError> {
        if self.pulses.is_empty() && self.defaults.pulse != UnresolvedPulse::default() {
            let c = self.defaults.pulse.clone();
            self.pulses.push(c);
        }
        self.pulses
            .iter_mut()
            .try_for_each(|pulse| pulse.resolve(&self.defaults))?;
        // NOTE: the sort_by flips the order to b, a
        // to put more ancient events at the front.
        // FIXME: we cannot remove this unwrap
        // unless we define Time as fully-ordered.
        self.pulses
            .sort_by(|a, b| b.time.partial_cmp(&a.time).unwrap());
        Ok(())
    }

    // NOTE: this function could output a resoled Graph
    // type and maybe save some extra work/moves.
    pub(crate) fn resolve(&mut self) -> Result<(), DemesError> {
        if self.demes.is_empty() {
            return Err(DemesError::DemeError(
                "no demes have been specified".to_string(),
            ));
        }
        std::mem::swap(&mut self.defaults, &mut self.input_defaults.defaults);
        self.defaults.validate()?;
        self.deme_map = self.build_deme_map()?;

        let mut resolved_demes = vec![];
        for deme in self.demes.iter_mut() {
            deme.resolve(&self.deme_map, &resolved_demes, &self.defaults)?;
            resolved_demes.push(deme.clone());
        }
        self.demes = resolved_demes;
        self.demes.iter().try_for_each(|deme| deme.validate())?;
        self.resolve_migrations()?;
        self.resolve_pulses()?;
        self.validate_migrations()?;

        match self.generation_time {
            Some(_) => (), //value.validate(DemesError::GraphError)?,
            None => {
                if matches!(self.time_units, TimeUnits::Generations) {
                    self.generation_time = Some(InputGenerationTime::from(1.));
                }
            }
        }
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

        if matches!(&self.time_units, TimeUnits::Generations) {
            if let Some(value) = self.generation_time {
                if !value.equals(1.0) {
                    return Err(DemesError::GraphError(
                        "time units are generations but generation_time != 1.0".to_string(),
                    ));
                }
            }
        }
        self.pulses
            .iter()
            .try_for_each(|pulse| pulse.validate(&self.deme_map, &self.demes))?;

        Ok(())
    }

    pub(crate) fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata
    }
}

/// A resolved demes Graph.
///
/// Instances of this type will be fully-resolved according to
/// the machine data model described
/// [here](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html#).
///
/// A graph cannot be directly initialized. See:
/// * [`load`](crate::load)
/// * [`loads`](crate::loads)
/// * [`GraphBuilder`](crate::GraphBuilder)
#[derive(Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, try_from = "UnresolvedGraph")]
pub struct Graph {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doi: Option<Vec<String>>,
    #[serde(default = "Metadata::default")]
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    metadata: Metadata,
    time_units: TimeUnits,
    generation_time: GenerationTime,
    pub(crate) demes: Vec<Deme>,
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

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string().unwrap())
    }
}

impl TryFrom<UnresolvedGraph> for Graph {
    type Error = DemesError;

    fn try_from(value: UnresolvedGraph) -> Result<Self, Self::Error> {
        let mut pulses = vec![];
        for p in value.pulses {
            pulses.push(Pulse::try_from(p)?);
        }
        let mut demes = vec![];
        for hdm_deme in value.demes.into_iter() {
            let deme = Deme::try_from(hdm_deme)?;
            demes.push(deme);
        }
        Ok(Self {
            description: value.description,
            doi: value.doi,
            metadata: value.metadata,
            time_units: value.time_units,
            generation_time: value
                .generation_time
                .ok_or_else(|| DemesError::GraphError("generation_time is unresolved".to_string()))?
                .try_into()?,
            demes,
            resolved_migrations: value.resolved_migrations,
            pulses,
            deme_map: value.deme_map,
        })
    }
}

impl Graph {
    pub(crate) fn new_from_str(yaml: &'_ str) -> Result<Self, DemesError> {
        let mut g: UnresolvedGraph = serde_yaml::from_str(yaml)?;
        g.resolve()?;
        g.validate()?;
        g.try_into()
    }

    #[cfg(feature = "json")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
    pub(crate) fn new_resolved_from_json_str(json: &'_ str) -> Result<Self, DemesError> {
        let mut g: UnresolvedGraph = serde_json::from_str(json)?;
        g.resolve()?;
        g.validate()?;
        g.try_into()
    }

    pub(crate) fn new_from_reader<T: Read>(reader: T) -> Result<Self, DemesError> {
        let mut g: UnresolvedGraph = serde_yaml::from_reader(reader)?;
        g.resolve()?;
        g.validate()?;
        g.try_into()
    }

    #[cfg(feature = "json")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
    pub(crate) fn new_from_json_reader<T: Read>(reader: T) -> Result<Self, DemesError> {
        let mut g: UnresolvedGraph = serde_json::from_reader(reader)?;
        g.resolve()?;
        g.validate()?;
        g.try_into()
    }

    pub(crate) fn new_resolved_from_str(yaml: &'_ str) -> Result<Self, DemesError> {
        let graph = Self::new_from_str(yaml)?;
        Ok(graph)
    }

    pub(crate) fn new_resolved_from_reader<T: Read>(reader: T) -> Result<Self, DemesError> {
        let graph = Self::new_from_reader(reader)?;
        Ok(graph)
    }

    #[cfg(feature = "json")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
    pub(crate) fn new_resolved_from_json_reader<T: Read>(reader: T) -> Result<Self, DemesError> {
        let graph = Self::new_from_json_reader(reader)?;
        Ok(graph)
    }

    /// The number of [`Deme`](crate::Deme) instances in the graph.
    pub fn num_demes(&self) -> usize {
        self.demes.len()
    }

    /// Obtain a reference to a [`Deme`](crate::Deme) by its name.
    ///
    /// # Returns
    ///
    /// `Some(&Deme)` if `name` exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// See [`here`](crate::SizeFunction).
    #[deprecated(note = "Use Graph::get_deme instead")]
    pub fn get_deme_from_name(&self, name: &str) -> Option<&Deme> {
        let id = DemeId::from(name);
        self.get_deme(id)
    }

    /// Get the [`Deme`](crate::Deme) at identifier `id`.
    ///
    /// # Parameters
    ///
    /// * `id`, the [`DemeId`] to fetch.
    ///
    /// # Panics
    ///
    /// * If either variant of [`DemeId`] refers to an invalid deme
    ///
    /// # Note
    ///
    /// See [`Graph::get_deme`] for a version that will not panic
    pub fn deme<'name, I: Into<DemeId<'name>>>(&self, id: I) -> &Deme {
        self.get_deme(id).unwrap()
    }

    /// Get the [`Deme`](crate::Deme) at identifier `id`.
    ///
    /// # Parameters
    ///
    /// * `id`, the [`DemeId`] to fetch.
    ///
    /// # Returns
    ///
    /// * `Some(&[`Deme`])` if `id` is valid
    /// * `None` otherwise
    pub fn get_deme<'name, I: Into<DemeId<'name>>>(&self, id: I) -> Option<&Deme> {
        match id.into() {
            DemeId::Index(i) => self.demes.get(i),
            DemeId::Name(name) => get_deme!(name, &self.deme_map, &self.demes),
        }
    }

    /// Get the [`Deme`](crate::Deme) instances via a slice.
    pub fn demes(&self) -> &[Deme] {
        &self.demes
    }

    /// Get the [`GenerationTime`](crate::GenerationTime) for the graph.
    pub fn generation_time(&self) -> GenerationTime {
        self.generation_time
    }

    /// Get the [`TimeUnits`](crate::TimeUnits) for the graph.
    pub fn time_units(&self) -> TimeUnits {
        self.time_units.clone()
    }

    /// Get the migration events for the graph.
    pub fn migrations(&self) -> &[AsymmetricMigration] {
        &self.resolved_migrations
    }

    /// Get the pulse events for the graph.
    pub fn pulses(&self) -> &[Pulse] {
        &self.pulses
    }

    /// Get a copy of the top-level [`Metadata`](crate::Metadata).
    pub fn metadata(&self) -> Option<Metadata> {
        if self.metadata.metadata.is_empty() {
            None
        } else {
            Some(self.metadata.clone())
        }
    }

    fn convert_to_generations_details(
        self,
        round: fn(Time, GenerationTime) -> Time,
    ) -> Result<Self, DemesError> {
        let mut converted = self;

        converted.demes.iter_mut().try_for_each(|deme| {
            deme.resolved_time_to_generations(converted.generation_time, round)
        })?;

        converted.pulses.iter_mut().try_for_each(|pulse| {
            pulse.resolved_time_to_generations(converted.generation_time, round)
        })?;

        converted
            .resolved_migrations
            .iter_mut()
            .try_for_each(|pulse| {
                pulse.resolved_time_to_generations(converted.generation_time, round)
            })?;

        converted.time_units = TimeUnits::Generations;
        converted.generation_time = GenerationTime::try_from(1.0)?;

        Ok(converted)
    }

    /// Convert the time units to generations.
    ///
    /// # Errors
    ///
    /// If the time unit of an event differs sufficiently in
    /// magnitude from the `generation_time`, it is possible
    /// that conversion results in epochs (or migration
    /// durations) of length zero, which will return an error.
    ///
    /// If any field is unresolved, an error will be returned.
    pub fn into_generations(self) -> Result<Self, DemesError> {
        self.into_generations_with(crate::time::to_generations)
    }

    /// Convert the time units to generations, rounding the output to an integer value.
    pub fn into_integer_generations(self) -> Result<Graph, DemesError> {
        self.into_generations_with(crate::time::round_time_to_integer_generations)
    }

    /// Convert the time units to generations with a callback to specify the conversion
    /// policy
    pub fn into_generations_with(
        self,
        with: fn(Time, GenerationTime) -> Time,
    ) -> Result<Graph, DemesError> {
        self.convert_to_generations_details(with)
    }

    /// Return a representation of the graph as a string.
    ///
    /// The format is in YAML and corresponds to the MDM
    /// representation of the data.
    ///
    /// # Error
    ///
    /// Will return an error if `serde_yaml::to_string`
    /// returns an error.
    pub fn as_string(&self) -> Result<String, DemesError> {
        match serde_yaml::to_string(self) {
            Ok(string) => Ok(string),
            Err(e) => Err(e.into()),
        }
    }

    /// Return a representation of the graph as a string.
    ///
    /// The format is in JSON and corresponds to the MDM
    /// representation of the data.
    ///
    /// # Error
    ///
    /// Will return an error if `serde_json::to_string`
    /// returns an error.
    #[cfg(feature = "json")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
    pub fn as_json_string(&self) -> Result<String, DemesError> {
        match serde_json::to_string(self) {
            Ok(string) => Ok(string),
            Err(e) => Err(e.into()),
        }
    }

    /// Return the most recent end time of any deme
    /// in the Graph.
    ///
    /// This function is useful to check if the most
    /// recent end time is greater than zero, meaning
    /// that the model ends at a time point ancestral to
    /// "now".
    pub fn most_recent_deme_end_time(&self) -> Time {
        let init = self.demes[0].end_time();
        self.demes
            .iter()
            .skip(1)
            .fold(init, |current_min, current_deme| {
                std::cmp::min(current_min, current_deme.end_time())
            })
    }

    /// Return the description field.
    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(x) => Some(x),
            None => None,
        }
    }

    /// Return an iterator over DOI information.
    pub fn doi(&self) -> impl Iterator<Item = &str> {
        DOIIterator::new(self)
    }

    /// Check if any epochs have non-integer
    /// `start_size` or `end_size`.
    ///
    /// # Returns
    ///
    /// * The deme name and epoch index where the first
    ///   non-integer value is encountered
    /// * None if non non-integer values are encountered
    pub fn has_non_integer_sizes(&self) -> Option<(&str, usize)> {
        for deme in &self.demes {
            for (i, epoch) in deme.epochs.iter().enumerate() {
                for size in [f64::from(epoch.start_size()), f64::from(epoch.end_size())] {
                    if size.is_finite() && size.fract() != 0.0 {
                        return Some((deme.name(), i));
                    }
                }
            }
        }
        None
    }

    fn epoch_start_end_size_rounding_details(
        old_size: DemeSize,
        rounding_fn: fn(f64) -> f64,
    ) -> Result<DemeSize, DemesError> {
        let size = f64::from(old_size);
        if size.is_finite() && size.fract() != 0.0 {
            let new_size = rounding_fn(size);
            if !new_size.is_finite() || new_size.fract() != 0.0 || new_size <= 0.0 {
                let msg = format!("invalid size after rounding: {new_size}");
                return Err(DemesError::EpochError(msg));
            }
            return new_size.try_into();
        }
        Ok(old_size)
    }

    fn round_epoch_start_end_sizes_with(
        self,
        rounding_fn: fn(f64) -> f64,
    ) -> Result<Self, DemesError> {
        let mut graph = self;

        for deme in &mut graph.demes {
            for epoch in &mut deme.epochs {
                epoch.start_size =
                    Graph::epoch_start_end_size_rounding_details(epoch.start_size, rounding_fn)?;
                epoch.end_size =
                    Graph::epoch_start_end_size_rounding_details(epoch.end_size, rounding_fn)?;
            }
        }

        Ok(graph)
    }

    /// Round all epoch start/end sizes to nearest integer value.
    ///
    /// # Returns
    ///
    /// A modified graph with rounded sizes.
    ///
    /// # Error
    ///
    /// * [`EpochError`](crate::DemesError::EpochError) if rounding
    ///   leads to a value of 0.
    ///
    /// # Note
    ///
    /// Rounding uses [f64::round](f64::round)
    pub fn into_integer_start_end_sizes(self) -> Result<Self, DemesError> {
        self.round_epoch_start_end_sizes_with(f64::round)
    }

    /// Obtain names of all demes in the graph.
    ///
    /// # Note
    ///
    /// These are ordered by a deme's index in the model.
    ///
    /// # Panics
    ///
    /// This function allocates space for the return value,
    /// which may panic upon out-of-memory.
    pub fn deme_names(&self) -> Box<[&str]> {
        self.demes
            .iter()
            .map(|deme| deme.name())
            .collect::<Vec<&str>>()
            .into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_display() {
        let t = Time::try_from(1.0).unwrap();
        let f = format!("{t}");
        assert_eq!(f, String::from("1"));
    }

    #[test]
    #[should_panic]
    fn test_time_validity() {
        let _ = Time::try_from(f64::NAN).unwrap();
    }

    #[test]
    fn test_newtype_compare_to_f64() {
        {
            let v = Time::try_from(100.0).unwrap();
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
        assert_eq!(g.migrations()[0].rate(), 0.25);
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
        assert_eq!(g.migrations()[0].rate(), 0.25);
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
        assert_eq!(g.migrations()[0].rate(), 0.25);
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
        assert_eq!(g.migrations()[0].rate(), 0.25);
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
        assert_eq!(g.migrations()[0].rate(), 0.25);
        assert_eq!(g.migrations()[1].rate(), 0.25);
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
        assert_eq!(g.migrations()[0].rate(), 0.25);
        assert_eq!(g.migrations()[1].rate(), 0.25);
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
        assert_eq!(g.pulses()[0].time(), 100.0);
    }
}

#[cfg(test)]
mod test_to_generations {
    #[test]
    fn test_raw_conversion() {
        let yaml = "
time_units: years
generation_time: 25
demes:
 - name: ancestor
   epochs:
    - start_size: 100
      end_time: 100
 - name: derived
   ancestors: [ancestor]
   epochs:
    - start_size: 100
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_generations().unwrap();
        let deme = converted.deme(0);
        assert_eq!(deme.end_time(), 4.0);
        let deme = converted.deme(1);
        assert_eq!(deme.start_time(), 4.0);
    }

    #[test]
    fn test_demelevel_default_epoch_conversion() {
        let yaml = "
time_units: years
generation_time: 25
demes:
 - name: ancestor
   defaults:
    epoch:
     end_time: 10
   epochs:
    - start_size: 100
 - name: derived
   ancestors: [ancestor]
   epochs:
    - start_size: 100
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_generations().unwrap();
        let deme = converted.deme(0);
        assert_eq!(deme.end_time(), 0.4);
        let deme = converted.deme(1);
        assert_eq!(deme.start_time(), 0.4);
    }

    #[test]
    fn test_pulse_conversion() {
        let yaml = "
time_units: years
generation_time: 25
demes:
 - name: one
   epochs:
    - start_size: 100
 - name: two
   epochs:
    - start_size: 100
pulses:
 - sources: [one]
   dest: two
   proportions: [0.25]
   time: 50
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_generations().unwrap();
        for p in converted.pulses().iter() {
            assert_eq!(p.time(), 2.0);
        }
    }

    #[test]
    fn test_default_pulse_conversion() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
 pulse:
  time: 50
demes:
 - name: one
   epochs:
    - start_size: 100
 - name: two
   epochs:
    - start_size: 100
pulses:
 - sources: [one]
   dest: two
   proportions: [0.25]
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_generations().unwrap();
        for p in converted.pulses().iter() {
            assert_eq!(p.time(), 2.0);
        }
    }

    #[test]
    fn test_migration_conversion() {
        let yaml = "
time_units: years
generation_time: 25
demes:
 - name: one
   epochs:
    - start_size: 100
 - name: two
   epochs:
    - start_size: 100
migrations:
 - demes: [one, two]
   rate: 0.25
   start_time: 50
   end_time: 10
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_generations().unwrap();
        for p in converted.migrations().iter() {
            assert_eq!(p.start_time(), 50.0 / 25.0);
            assert_eq!(p.end_time(), 10.0 / 25.0);
        }
    }

    #[test]
    fn test_default_migration_conversion() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
 migration:
  start_time: 50
  end_time: 10
demes:
 - name: one
   epochs:
    - start_size: 100
 - name: two
   epochs:
    - start_size: 100
migrations:
 - demes: [one, two]
   rate: 0.25
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_generations().unwrap();
        for p in converted.migrations().iter() {
            assert_eq!(p.start_time(), 50.0 / 25.0);
            assert_eq!(p.end_time(), 10.0 / 25.0);
        }
    }

    #[test]
    fn test_toplevel_default_epoch_conversion() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
 deme:
   start_time: 100
demes:
 - name: ancestor
   start_time: .inf
   epochs:
    - start_size: 100
      end_time: 10
 - name: derived
   ancestors: [ancestor]
   epochs:
    - start_size: 100
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_generations().unwrap();
        assert!(matches!(
            converted.time_units(),
            super::TimeUnits::Generations
        ));
        let deme = converted.deme(0);
        assert_eq!(deme.end_time(), 10.0 / 25.0);
        let deme = converted.deme(1);
        assert_eq!(deme.start_time(), 100.0 / 25.0);
    }

    #[test]
    #[should_panic]
    fn test_raw_conversion_to_zero_length_epoch() {
        let yaml = "
time_units: years
generation_time: 1e300
demes:
 - name: ancestor
   epochs:
    - start_size: 100
      end_time: 1e-200
 - name: derived
   ancestors: [ancestor]
   epochs:
    - start_size: 100
";
        let g = crate::loads(yaml).unwrap();

        let _ = g.into_generations().unwrap();
    }
}

#[cfg(test)]
mod test_to_integer_generations {
    #[test]
    fn test_demelevel_default_epoch_conversion() {
        let yaml = "
time_units: years
generation_time: 25
demes:
 - name: ancestor
   defaults:
    epoch:
     end_time: 103
   epochs:
    - start_size: 100
 - name: derived
   ancestors: [ancestor]
   epochs:
    - start_size: 100
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_integer_generations().unwrap();
        let deme = converted.deme(0);
        assert_eq!(deme.end_time(), (103_f64 / 25.0).round());
        let deme = converted.deme(1);
        assert_eq!(deme.start_time(), (103_f64 / 25.0).round());

        let g2 = serde_yaml::to_string(&converted).unwrap();
        let converted_from_str = crate::loads(&g2).unwrap();
        assert_eq!(converted, converted_from_str);
    }

    #[test]
    #[should_panic]
    fn test_conversion_to_zero_length_epoch() {
        let yaml = "
time_units: years
description: rounding results in epochs of length zero
generation_time: 25
demes:
 - name: ancestor
   epochs:
    - start_size: 100
      end_time: 10
 - name: derived
   ancestors: [ancestor]
   epochs:
    - start_size: 100
";
        let g = crate::loads(yaml).unwrap();

        g.into_integer_generations().unwrap();
    }

    #[test]
    fn test_demelevel_epoch_conversion_non_integer_input_times() {
        let yaml = "
time_units: generations
demes:
 - name: ancestor
   defaults:
    epoch:
     end_time: 10.6
   epochs:
    - start_size: 100
 - name: derived
   ancestors: [ancestor]
   epochs:
    - start_size: 100
";
        let g = crate::loads(yaml).unwrap();

        let converted = g.into_integer_generations().unwrap();
        let deme = converted.deme(0);
        assert_eq!(deme.end_time(), 10.6_f64.round());
        let deme = converted.deme(1);
        assert_eq!(deme.start_time(), 10.6_f64.round());
    }

    #[test]
    #[should_panic]
    fn invalid_second_epoch_length_when_integer_rounded() {
        let yaml = "
time_units: years
description:
  50/1000 = 0.05, rounds to zero.
  Thus, the second epoch has length zero.
generation_time: 1000.0
demes:
 - name: A
   epochs:
    - start_size: 200
      end_time: 50
    - start_size: 100
";
        let graph = crate::loads(yaml).unwrap();
        let _ = graph.into_integer_generations().unwrap();
    }
}
