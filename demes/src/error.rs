/// Error type for this crate.
///
/// The enum fields correspond to
/// the different parts of a [Graph](crate::Graph)
/// defined by the
/// [specification](https://popsim-consortium.github.io/demes-spec-docs/main/introduction.html).
///
/// # Example
///
/// This input is incorrect because the epoch fails
/// to define `start_size` or `end_size`.
/// Attempting to generate a [Graph](crate::Graph)
/// gives [`DemesError::EpochError`](crate::DemesError::EpochError).
///
/// ```
/// let yaml = "
/// time_units: generations
/// demes:
///  - name: A
///    epochs:
///     - end_time: 100
/// ";
/// assert!(matches!(demes::loads(yaml), Err(demes::DemesError::EpochError(_))));
/// ```
// cbindgen:no-export
#[derive(Debug)]
#[non_exhaustive]
pub enum DemesError {
    /// Errors related to demes
    // #[error("{0:?}")]
    DemeError(String),
    // #[error("{0:?}")]
    /// Errors related to epochs
    EpochError(String),
    // #[error("{0:?}")]
    /// Top-level errors.
    GraphError(String),
    // #[error("{0:?}")]
    /// Errors related to migrations
    MigrationError(String),
    // #[error("{0:?}")]
    /// Errors related to pulses
    PulseError(String),
    /// Errors coming from `serde_yaml`.
    YamlError(serde_yaml::Error),
    #[cfg(feature = "json")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
    /// Errors coming from `serde_json`.
    JsonError(serde_json::Error),
    #[cfg(feature = "toml")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "toml")))]
    /// Errors coming from `toml` during deserialization.
    TomlDeError(toml::de::Error),
    /// Errors related to low-level types
    // #[error("{0:?}")]
    ValueError(String),
    /// IO errors from the rust standard library
    IOerror(std::io::Error),
}

impl From<serde_yaml::Error> for DemesError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::YamlError(value)
    }
}

impl From<std::io::Error> for DemesError {
    fn from(value: std::io::Error) -> Self {
        Self::IOerror(value)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for DemesError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value)
    }
}

#[cfg(feature = "toml")]
impl From<toml::de::Error> for DemesError {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlDeError(value)
    }
}

impl std::fmt::Display for DemesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemesError::DemeError(e) => write!(f, "deme error: {}", e),
            DemesError::EpochError(e) => write!(f, "epoch error: {}", e),
            DemesError::GraphError(e) => write!(f, "graph error: {}", e),
            DemesError::MigrationError(e) => write!(f, "migration error: {}", e),
            DemesError::PulseError(e) => write!(f, "pulse error: {}", e),
            DemesError::YamlError(e) => write!(f, "yaml error: {}", e),
            DemesError::ValueError(e) => write!(f, "value error: {}", e),
            DemesError::IOerror(e) => write!(f, "io error: {}", e),
            #[cfg(feature = "json")]
            DemesError::JsonError(e) => write!(f, "JSON error: {e:?}"),
            #[cfg(feature = "toml")]
            DemesError::TomlDeError(e) => write!(f, "TOML deserialization error: {e:?}"),
        }
    }
}

impl std::error::Error for DemesError {}
