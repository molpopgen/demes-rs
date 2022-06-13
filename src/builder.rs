use itertools::Itertools;

use crate::specification::Deme;
use crate::specification::DemeDefaults;
use crate::specification::Epoch;
use crate::specification::EpochData;
use crate::specification::Graph;
use crate::specification::Proportion;
use crate::specification::Time;
use crate::specification::TimeUnits;
use crate::DemesError;

pub struct GraphBuilder {
    graph: Graph,
}

impl GraphBuilder {
    // public API
    pub fn new(time_units: TimeUnits) -> Self {
        Self {
            graph: Graph::new_from_time_units(time_units),
        }
    }

    pub fn resolve(self) -> Result<Graph, DemesError> {
        let mut builder = self;
        builder.graph.resolve()?;
        Ok(builder.graph)
    }

    pub fn add_deme(&mut self, name: &str, options: Option<DemeOptions>) -> Result<(), DemesError> {
        let data = crate::specification::DemeData::new_with_name(name);
        let mut ptr = Deme::new_from_deme_data(data);
        ptr.validate_name()?;
        apply_options(&mut ptr, options)?;
        self.graph.add_deme(ptr);
        Ok(())
    }
}

fn apply_options(deme: &mut Deme, options: Option<DemeOptions>) -> Result<(), DemesError> {
    match options {
        Some(options) => {
            let mut mut_self_borrow = deme.borrow_mut();
            match options.description {
                Some(description) => mut_self_borrow.set_description(Some(&description)),
                None => mut_self_borrow.set_description(None),
            }
            mut_self_borrow.set_ancestors(options.ancestors);
            match options.proportions {
                Some(proportions) => mut_self_borrow.set_proportions(Some(proportions.proportions)),
                None => mut_self_borrow.set_proportions(None),
            }
            match options.epochs {
                Some(epochs) => {
                    mut_self_borrow.set_epochs(
                        epochs
                            .into_iter()
                            .map(Epoch::new_from_epoch_data)
                            .collect_vec(),
                    );
                }
                None => (),
            }
            mut_self_borrow.set_start_time(options.start_time)?;
            match options.defaults {
                Some(defaults) => mut_self_borrow.set_defaults(defaults),
                None => (),
            }
        }
        None => {}
    }
    Ok(())
}

// NOTE: this is starting to be
// a wholesale dup of specification::DemeData
// "Fixing" this dup is tricky:
// * See my notes on Google Keep
// * We'd need to separate out epochs from epoch_data
//   and their visibility, making sure that the
//   ..Default::default() pattern "just works" for client
//   code.
#[derive(Default)]
pub struct DemeOptions {
    pub description: Option<String>,
    pub ancestors: Option<Vec<String>>,
    pub proportions: Option<ProportionsProxy>,
    pub epochs: Option<Vec<EpochData>>,
    pub start_time: Option<Time>,
    pub defaults: Option<DemeDefaults>,
}

pub struct ProportionsProxy {
    proportions: Vec<Proportion>,
}

impl TryFrom<Vec<f64>> for ProportionsProxy {
    type Error = DemesError;
    fn try_from(value: Vec<f64>) -> Result<Self, Self::Error> {
        let mut proportions = vec![];
        for v in value.into_iter() {
            let p = Proportion::try_from(v)?;
            proportions.push(p);
        }
        Ok(Self { proportions })
    }
}

impl From<Vec<Proportion>> for ProportionsProxy {
    fn from(proportions: Vec<Proportion>) -> Self {
        Self { proportions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specification::DemeSize;

    #[test]
    #[should_panic]
    fn new_builder() {
        let b = GraphBuilder::new(TimeUnits::Generations);
        b.resolve().unwrap();
    }

    #[test]
    fn add_deme() {
        let mut b = GraphBuilder::new(TimeUnits::Generations);
        b.add_deme("CEU", None).unwrap();
    }

    #[test]
    fn add_deme_with_epochs() {
        let mut b = GraphBuilder::new(TimeUnits::Generations);
        let edata = EpochData {
            start_size: Some(DemeSize::try_from(100.0).unwrap()),
            ..Default::default()
        };
        let defaults = DemeOptions {
            epochs: Some(vec![edata]),
            ..Default::default()
        };
        b.add_deme("CEU", Some(defaults)).unwrap();
        let _graph = b.resolve().unwrap();
    }

    #[test]
    #[should_panic]
    fn add_deme_invalid_name() {
        let mut b = GraphBuilder::new(TimeUnits::Generations);
        let _d = b.add_deme("12", None).unwrap();
    }

    #[test]
    fn use_f64_for_proportions() {
        let _ = DemeOptions {
            proportions: Some(vec![0.5, 0.5].try_into().unwrap()),
            ..Default::default()
        };
    }

    #[test]
    fn use_proportion_for_proportions() {
        let p = Proportion::try_from(0.5).unwrap();
        let _ = DemeOptions {
            proportions: Some(vec![p, p].into()),
            ..Default::default()
        };
    }

    #[test]
    fn builder_deme_defaults() {
        let defaults = DemeDefaults {
            epoch: EpochData {
                end_size: Some(DemeSize::try_from(100.).unwrap()),
                ..Default::default()
            },
        };
        let deme = DemeOptions {
            defaults: Some(defaults),
            ..Default::default()
        };
        let mut b = GraphBuilder::new(TimeUnits::Generations);
        b.add_deme("YRB", Some(deme)).unwrap();
        b.resolve().unwrap();
    }
}
