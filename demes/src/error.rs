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
    DemeError(String),
    /// Errors related to epochs
    EpochError(String),
    /// Top-level errors.
    GraphError(String),
    /// Errors related to migrations
    MigrationError(String),
    /// Errors related to pulses
    PulseError(String),
    /// Errors coming from `serde_yaml`.
    YamlError(OpaqueYamlError),
    #[cfg(feature = "json")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
    /// Errors coming from `serde_json`.
    JsonError(OpaqueJSONError),
    #[cfg(feature = "toml")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "toml")))]
    /// Errors coming from `toml` during deserialization.
    TomlDeError(OpaqueTOMLError),
    /// Errors related to low-level types
    ValueError(String),
    /// IO errors from the rust standard library
    IOerror(OpaqueIOError),
}

#[derive(Debug)]
#[repr(transparent)]
pub struct OpaqueYamlError(pub(crate) serde_yaml::Error);

#[derive(Debug)]
#[repr(transparent)]
pub struct OpaqueIOError(pub(crate) std::io::Error);

#[cfg(feature = "json")]
#[derive(Debug)]
#[repr(transparent)]
pub struct OpaqueJSONError(pub(crate) serde_json::Error);

#[cfg(feature = "toml")]
#[derive(Debug)]
#[repr(transparent)]
pub struct OpaqueTOMLError(pub(crate) toml::de::Error);

impl std::fmt::Display for DemesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemesError::DemeError(e) => write!(f, "deme error: {}", e),
            DemesError::EpochError(e) => write!(f, "epoch error: {}", e),
            DemesError::GraphError(e) => write!(f, "graph error: {}", e),
            DemesError::MigrationError(e) => write!(f, "migration error: {}", e),
            DemesError::PulseError(e) => write!(f, "pulse error: {}", e),
            DemesError::YamlError(e) => write!(f, "yaml error: {}", e.0),
            DemesError::ValueError(e) => write!(f, "value error: {}", e),
            DemesError::IOerror(e) => write!(f, "io error: {}", e.0),
            #[cfg(feature = "json")]
            DemesError::JsonError(e) => write!(f, "JSON error: {e:?}"),
            #[cfg(feature = "toml")]
            DemesError::TomlDeError(e) => write!(f, "TOML deserialization error: {e:?}"),
        }
    }
}

impl std::error::Error for DemesError {}
