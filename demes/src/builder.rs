use thiserror::Error;

use crate::specification::Graph;
use crate::specification::GraphDefaults;
use crate::specification::UnresolvedDeme;
use crate::specification::UnresolvedDemeHistory;
use crate::specification::UnresolvedEpoch;
use crate::specification::UnresolvedGraph;
use crate::DemesError;
use crate::InputGenerationTime;
use crate::InputMigrationRate;
use crate::InputProportion;
use crate::InputTime;
use crate::TimeUnits;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum BuilderError {
    #[error("{0:?}")]
    SerdeYamlError(#[from] serde_yaml::Error),
}

/// This type allows building a [`Graph`](crate::Graph) using code
/// rather then using text input.
///
/// # Notes
///
/// * A "builder" in rust will never be as convenient
///   as one in, say, Python or Juilia.
///   The lack of a rust REPL and the strong type checking
///   are the primary reasons.
/// * All error checks are delayed until resolution.
pub struct GraphBuilder {
    graph: UnresolvedGraph,
    metadata: Option<crate::Metadata>,
}

impl GraphBuilder {
    /// Constructor
    ///
    /// # Returns
    ///
    /// This function returns an "builder" containing an unresolved
    /// [`Graph`](crate::Graph).
    pub fn new(
        time_units: TimeUnits,
        generation_time: Option<InputGenerationTime>,
        defaults: Option<GraphDefaults>,
    ) -> Self {
        Self {
            graph: UnresolvedGraph::new(time_units, generation_time, defaults),
            metadata: None,
        }
    }

    /// Construct a builder with time units in generations.
    ///
    /// This function works by calling [`GraphBuilder::new`](crate::GraphBuilder::new).
    pub fn new_generations(defaults: Option<GraphDefaults>) -> Self {
        Self {
            graph: UnresolvedGraph::new(TimeUnits::Generations, None, defaults),
            metadata: None,
        }
    }

    /// Add a [`Deme`](crate::Deme) to the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history, Some("this is deme A"));
    /// b.resolve().unwrap();
    /// ```
    ///
    /// # Notes
    pub fn add_deme(
        &mut self,
        name: &str,
        epochs: Vec<UnresolvedEpoch>,
        history: UnresolvedDemeHistory,
        description: Option<&str>,
    ) {
        let ptr = UnresolvedDeme::new_via_builder(name, epochs, history, description);
        self.graph.add_deme(ptr);
    }

    /// Add an asymmetric migration
    ///
    /// # Examples
    ///
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// b.add_asymmetric_migration(Some("A"),
    ///                            Some("B"),
    ///                            Some(1e-4.into()),
    ///                            None, // Using None for the times
    ///                                  // will mean continuous migration for the
    ///                                  // duration for which the demes coexist.
    ///                            None);
    /// b.resolve().unwrap();
    /// ```
    pub fn add_asymmetric_migration<S: ToString, D: ToString>(
        &mut self,
        source: Option<S>,
        dest: Option<D>,
        rate: Option<InputMigrationRate>,
        start_time: Option<InputTime>,
        end_time: Option<InputTime>,
    ) {
        self.add_migration::<String, _, _>(None, source, dest, rate, start_time, end_time);
    }

    /// Add a symmetric migration
    ///
    /// # Examples
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// b.add_symmetric_migration(Some(&["A", "B"]),
    ///                           Some(1e-4.into()),
    ///                           None, // Using None for the times
    ///                                 // will mean continuous migration for the
    ///                                 // duration for which the demes coexist.
    ///                           None);
    /// b.resolve().unwrap();
    /// ```
    pub fn add_symmetric_migration<D: ToString>(
        &mut self,
        demes: Option<&[D]>,
        rate: Option<InputMigrationRate>,
        start_time: Option<InputTime>,
        end_time: Option<InputTime>,
    ) {
        self.add_migration::<_, String, String>(demes, None, None, rate, start_time, end_time);
    }

    /// Add a migration to the graph.
    ///
    /// # Note
    ///
    /// This function can be inconvenient due to the generics.
    /// Prefer [`add_symmetric_migration`](crate::GraphBuilder::add_symmetric_migration)
    /// or [`add_asymmetric_migration`](crate::GraphBuilder::add_asymmetric_migration).
    ///
    /// # Examples
    ///
    /// ## Adding an asymmetric migration
    ///
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// b.add_migration::<String, _, _>(None,
    ///                   Some("A"),
    ///                   Some("B"),
    ///                   Some(1e-4.into()),
    ///                   None, // Using None for the times
    ///                         // will mean continuous migration for the
    ///                         // duration for which the demes coexist.
    ///                   None);
    /// b.resolve().unwrap();
    /// ```
    ///
    /// ## Adding a symmetric migration
    ///
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// b.add_migration::<_, String, String>(Some(&["A", "B"]),
    ///                   None,
    ///                   None,
    ///                   Some(1e-4.into()),
    ///                   None, // Using None for the times
    ///                         // will mean continuous migration for the
    ///                         // duration for which the demes coexist.
    ///                   None);
    /// b.resolve().unwrap();
    /// ```
    pub fn add_migration<D: ToString, S: ToString, E: ToString>(
        &mut self,
        demes: Option<&[D]>,
        source: Option<S>,
        dest: Option<E>,
        rate: Option<InputMigrationRate>,
        start_time: Option<InputTime>,
        end_time: Option<InputTime>,
    ) {
        let demes = demes.map(|value| value.iter().map(|v| v.to_string()).collect::<Vec<_>>());
        let source = source.map(|value| value.to_string());
        let dest = dest.map(|value| value.to_string());
        self.graph
            .add_migration(demes, source, dest, rate, start_time, end_time);
    }

    /// Add an [`UnresolvedPulse`](crate::UnresolvedPulse) to the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// b.add_pulse(Some(&["A"]),
    ///             Some("B"),
    ///             Some(demes::InputTime::from(50.0)),
    ///             Some(vec![demes::InputProportion::from(0.5)]));
    /// b.resolve().unwrap();
    /// ```
    pub fn add_pulse(
        &mut self,
        sources: Option<&[&str]>,
        dest: Option<&str>,
        time: Option<InputTime>,
        proportions: Option<Vec<InputProportion>>,
    ) {
        let sources = sources.map(|value| value.iter().map(|v| v.to_string()).collect::<Vec<_>>());
        let dest = dest.map(|value| value.to_string());
        self.graph.add_pulse(sources, dest, time, proportions);
    }

    /// Generate and return a resolved [`Graph`](crate::Graph).
    ///
    /// # Errors
    ///
    /// Returns [`DemesError'](crate::DemesError) if any
    /// of the data are invalid.
    pub fn resolve(self) -> Result<Graph, DemesError> {
        let mut builder = self;
        match builder.metadata {
            None => (),
            Some(m) => builder.graph.set_metadata(m),
        }
        builder.graph.resolve()?;
        builder.graph.try_into()
    }

    /// Set top-level metadata
    ///
    /// # Parameters
    ///
    /// * `metadata`: the metadata type
    ///
    /// # Note
    ///
    /// Repeated calls will overwrite existing metadata.
    ///
    /// # Errors
    ///
    /// * [`BuilderError`] if serialization to YAML fails.
    ///
    /// # Example
    ///
    /// ```
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct MyMetaData {
    ///    foo: i32,
    ///    bar: String
    /// }
    /// # let mut builder = demes::GraphBuilder::new_generations(None);
    /// builder.set_toplevel_metadata(&MyMetaData{foo: 3, bar: "string".to_owned()}).unwrap();
    /// ```
    pub fn set_toplevel_metadata<T: serde::Serialize>(
        &mut self,
        metadata: &T,
    ) -> Result<(), BuilderError> {
        let yaml = serde_yaml::to_string(metadata)?;
        let metadata: crate::Metadata = serde_yaml::from_str(&yaml)?;
        self.metadata = Some(metadata);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specification::DemeDefaults;
    use crate::InputDemeSize;

    #[test]
    #[should_panic]
    fn new_builder() {
        let b = GraphBuilder::new(TimeUnits::Generations, None, None);
        b.resolve().unwrap();
    }

    #[test]
    fn add_deme_with_epochs() {
        let mut b = GraphBuilder::new_generations(Some(GraphDefaults::default()));
        let edata = UnresolvedEpoch {
            start_size: Some(InputDemeSize::from(100.0)),
            ..Default::default()
        };
        b.add_deme("CEU", vec![edata], UnresolvedDemeHistory::default(), None);
        let _graph = b.resolve().unwrap();
    }

    #[test]
    fn use_proportion_for_proportions() {
        let p = InputProportion::from(0.5);
        let _ = UnresolvedDemeHistory {
            proportions: Some(vec![p, p]),
            ..Default::default()
        };
    }

    #[test]
    fn builder_deme_defaults() {
        let defaults = DemeDefaults {
            epoch: UnresolvedEpoch {
                end_size: Some(InputDemeSize::from(100.)),
                ..Default::default()
            },
        };
        let history = UnresolvedDemeHistory {
            defaults,
            ..Default::default()
        };
        let mut b = GraphBuilder::new_generations(None);
        b.add_deme("YRB", vec![], history, None);
        b.resolve().unwrap();
    }
}
