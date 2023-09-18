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
use crate::UnresolvedMigration;

/// Error type raised by [`GraphBuilder`]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum BuilderError {
    /// Error type when defaults fail to serialize
    #[error(transparent)]
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

/// Build a symmetric migration epoch.
///
/// # Examples
///
/// See [`GraphBuilder::add_migration`].
#[derive(Clone, Default)]
pub struct SymmetricMigrationBuilder {
    /// The demes that are involved
    pub demes: Option<Vec<String>>,
    /// The start time
    pub start_time: Option<InputTime>,
    /// The end time
    pub end_time: Option<InputTime>,
    /// The symmetric migration rate.
    pub rate: Option<InputMigrationRate>,
}

impl SymmetricMigrationBuilder {
    /// Set the demes
    pub fn set_demes<D, A>(self, d: D) -> Self
    where
        D: std::ops::Deref<Target = [A]>,
        A: AsRef<str>,
    {
        Self {
            demes: Some(d.iter().map(|a| a.as_ref().to_owned()).collect::<Vec<_>>()),
            ..self
        }
    }

    /// Set the start time
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

impl From<SymmetricMigrationBuilder> for UnresolvedMigration {
    fn from(value: SymmetricMigrationBuilder) -> Self {
        Self {
            demes: value.demes,
            start_time: value.start_time,
            end_time: value.end_time,
            rate: value.rate,
            ..Default::default()
        }
    }
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
    pub fn add_deme<D: std::ops::Deref<Target = [UnresolvedEpoch]>>(
        &mut self,
        name: &str,
        epochs: D,
        history: UnresolvedDemeHistory,
        description: Option<&str>,
    ) {
        let ptr = UnresolvedDeme::new_via_builder(name, epochs.to_owned(), history, description);
        self.graph.add_deme(ptr);
    }

    /// Add a migration to the graph.
    ///
    /// # Examples
    ///
    /// ## Adding an asymmetric migration
    ///
    /// See [`UnresolvedMigration`].
    ///
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// let migration = demes::UnresolvedMigration::default().set_source("A").set_dest("B").set_rate(1e-4);
    /// b.add_migration(migration);
    /// b.resolve().unwrap();
    /// ```
    ///
    /// ## Adding a symmetric migration
    ///
    /// See [`UnresolvedMigration`].
    ///
    /// ```
    /// let start_size = demes::InputDemeSize::from(100.);
    /// let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// let history = demes::UnresolvedDemeHistory::default();
    /// let mut b = demes::GraphBuilder::new_generations(None);
    /// b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// let migration = demes::UnresolvedMigration::default().set_demes(["A","B"].as_slice()).set_rate(1e-4);
    /// b.add_migration(migration);
    /// b.resolve().unwrap();
    /// ```
    ///
    /// We can also use a `Vec` instead of an array:
    ///
    /// ```
    /// # let start_size = demes::InputDemeSize::from(100.);
    /// # let epoch = demes::UnresolvedEpoch{start_size: Some(start_size), ..Default::default()};
    /// # let history = demes::UnresolvedDemeHistory::default();
    /// # let mut b = demes::GraphBuilder::new_generations(None);
    /// # b.add_deme("A", vec![epoch], history.clone(), Some("this is deme A"));
    /// # b.add_deme("B", vec![epoch], history, Some("this is deme B"));
    /// let migration = demes::UnresolvedMigration::default().set_demes(vec!["A","B"]).set_rate(1e-4);
    /// b.add_migration(migration);
    /// # b.resolve().unwrap();
    /// ```
    pub fn add_migration(&mut self, migration: UnresolvedMigration) {
        self.graph.add_migration(migration);
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
    ///             Some(50.0),
    ///             Some([0.5].as_slice()));
    /// b.resolve().unwrap();
    /// ```
    pub fn add_pulse<
        T: Into<InputTime>,
        P: Into<InputProportion> + Copy,
        D: std::ops::Deref<Target = [P]>,
    >(
        &mut self,
        sources: Option<&[&str]>,
        dest: Option<&str>,
        time: Option<T>,
        proportions: Option<D>,
    ) {
        let sources = sources.map(|value| value.iter().map(|v| v.to_string()).collect::<Vec<_>>());
        let dest = dest.map(|value| value.to_string());
        let time = time.map(|t| t.into());
        let proportions = proportions.map(|s| {
            s.iter()
                .map(|p| (*p).into())
                .collect::<Vec<InputProportion>>()
        });
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
