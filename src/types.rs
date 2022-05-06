use crate::DemesError;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

fn default_none_for<T>() -> Option<T> {
    None
}

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

    fn contains_start_time(&self, other: StartTime) -> bool {
        self.contains(other)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SizeFunction {
    #[serde(skip)]
    NONE,
    #[serde(rename = "constant")]
    CONSTANT,
    #[serde(rename = "exponential")]
    EXPONENTIAL,
    #[serde(rename = "linear")]
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
#[serde(deny_unknown_fields)]
pub struct Epoch {
    #[serde(default = "default_none_for::<EndTime>")]
    end_time: Option<EndTime>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    #[serde(default = "default_none_for::<DemeSize>")]
    start_size: Option<DemeSize>,
    // NOTE: the Option is for input. An actual value must be put in via resolution.
    #[serde(default = "default_none_for::<DemeSize>")]
    end_size: Option<DemeSize>,
    #[serde(default = "SizeFunction::default")]
    size_function: SizeFunction,
    #[serde(default = "CloningRate::default")]
    cloning_rate: CloningRate,
    #[serde(default = "SelfingRate::default")]
    selfing_rate: SelfingRate,
}

impl Default for Epoch {
    fn default() -> Self {
        Self {
            end_time: default_none_for::<EndTime>(),
            start_size: default_none_for::<DemeSize>(),
            end_size: default_none_for::<DemeSize>(),
            size_function: SizeFunction::default(),
            cloning_rate: CloningRate::default(),
            selfing_rate: SelfingRate::default(),
        }
    }
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
            mut_borrowed_self.start_time = StartTime::try_from(
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
            )?; // Err if cannot convert
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

    fn resolve_first_epoch_sizes(&mut self) -> Result<Option<DemeSize>, DemesError> {
        let mut self_borrow = self.0.borrow_mut();
        let epoch_sizes = {
            let temp_epoch = self_borrow.epochs.get_mut(0).unwrap();
            if temp_epoch.start_size.is_none() && temp_epoch.end_size.is_none() {
                return Err(DemesError::EpochError(format!(
                    "first epoch of deme {} must define one or both of start_size and end_size",
                    self.name()
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
                    self.name(), f64::from(epoch_sizes.0.unwrap()), f64::from(epoch_sizes.1.unwrap()),
                );
            return Err(DemesError::EpochError(msg));
        }
        Ok(epoch_sizes.1)
    }

    fn resolve_sizes(&mut self) -> Result<(), DemesError> {
        let mut last_end_size = self.resolve_first_epoch_sizes()?;
        for epoch in self.0.borrow_mut().epochs.iter_mut().skip(1) {
            if epoch.start_size.is_none() {
                epoch.start_size = last_end_size;
            }
            if epoch.end_size.is_none() {
                epoch.end_size = epoch.start_size;
            }
            last_end_size = epoch.end_size;
        }
        Ok(())
    }

    fn check_empty_epochs(&mut self, defaults: &Option<GraphDefaults>) -> Result<(), DemesError> {
        if !self.0.borrow().epochs.is_empty() {
            return Ok(());
        }

        match defaults.as_ref() {
            Some(&graph_defaults) => match graph_defaults.epoch.as_ref() {
                Some(&epoch_defaults) => {
                    let e = Epoch {
                        start_size: Some(epoch_defaults.start_size),
                        ..Default::default()
                    };
                    self.0.borrow_mut().epochs.push(e);
                }
                None => {
                    return Err(DemesError::TopLevelError(
                        "missing default start_size for epoch".to_string(),
                    ));
                }
            },
            None => {
                return Err(DemesError::TopLevelError(
                    "missing top-level defaults".to_string(),
                ));
            }
        }

        Ok(())
    }

    // Make the internal data match the MDM spec
    fn resolve(
        &mut self,
        deme_map: &DemeMap,
        defaults: Option<GraphDefaults>,
    ) -> Result<(), DemesError> {
        self.check_empty_epochs(&defaults)?;
        assert!(self.0.borrow().ancestor_map.is_empty());
        self.resolve_times(deme_map)?;
        self.resolve_sizes()?;
        self.0
            .borrow_mut()
            .epochs
            .iter_mut()
            .try_for_each(|e| e.resolve())?;

        let mut ancestor_map = DemeMap::default();
        let mut mut_self_borrow = self.0.borrow_mut();
        for ancestor in &mut_self_borrow.ancestors {
            ancestor_map.insert(ancestor.clone(), deme_map.get(ancestor).unwrap().clone());
        }
        mut_self_borrow.ancestor_map = ancestor_map;
        Ok(())
    }

    fn validate(&self) -> Result<(), DemesError> {
        if self.0.borrow().epochs.is_empty() {
            return Err(DemesError::DemeError(format!(
                "no epochs for deme {}",
                self.name()
            )));
        }
        self.0
            .borrow()
            .epochs
            .iter()
            .try_for_each(|e| e.validate())?;

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TimeUnits {
    #[serde(rename = "generations")]
    GENERATIONS,
}

impl Default for TimeUnits {
    fn default() -> Self {
        TimeUnits::GENERATIONS
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct EpochDefaults {
    start_size: DemeSize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct GraphDefaults {
    #[serde(default = "default_none_for::<EpochDefaults>")]
    epoch: Option<EpochDefaults>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    #[serde(default = "default_none_for::<GraphDefaults>")]
    #[serde(skip_serializing)]
    defaults: Option<GraphDefaults>,
    #[serde(default = "TimeUnits::default")]
    time_units: TimeUnits,
    demes: Vec<Deme>,
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

    pub(crate) fn resolve(&mut self) -> Result<(), DemesError> {
        self.deme_map = self.build_deme_map()?;

        self.demes
            .iter_mut()
            .try_for_each(|deme| deme.resolve(&self.deme_map, self.defaults))?;
        self.demes.iter().try_for_each(|deme| deme.validate())?;
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
}
