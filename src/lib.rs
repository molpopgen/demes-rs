//! rust support for
//! [demes](https://popsim-consortium.github.io/demes-spec-docs).
//!
//! # Introduction
//!
//! This crate provides:
//!
//! * Support for reading `YAML` descriptions of `demes` models.
//!   See [`loads`](crate::loads) and [`load`](crate::load).
//! * Support for building a demes model using `rust` code.
//!   See [`GraphBuilder`](crate::GraphBuilder).
//!
//! The output of any of these operations is a fully-resolved
//! [`Graph`](crate::Graph).
//!
//! ## More information
//!
//! * See [here](https://popsim-consortium.github.io/demes-spec-docs/main/introduction.html#) for
//! an overview of `demes`.
//!
//! ## Technical details
//!
//! * `YAML` and [`GraphBuilder`](crate::GraphBuilder) inputs
//!   support the Human Data Model (HDM) described in the
//!   demes
//!   [specification](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html)
//! * A [`Graph`] is fully-resolved according to the Machine
//!   Data Model (MDM) described in the
//!   [specification](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html).
//!   
//! # Known limitations
//!
//! * There are currently no convenience functions for exporting
//!   a [`Graph`](crate::Graph) back into `YAML`.
//!   However, this task is easily done via [serde_yaml::to_string].

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

mod macros;

mod builder;
mod error;
mod specification;

use std::io::Read;

pub use builder::GraphBuilder;
pub use error::DemesError;
pub use specification::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build a [`Graph`](crate::Graph) from an in-memory [`str`](std::primitive::str).
///
/// # Errors
///
/// Returns [`DemesError`](crate::DemesError) in the event of invalid input.
///
/// # Examples
///
/// ```
/// let yaml = "
/// time_units: generations
/// demes:
///  - name: ancestor
///    epochs:
///     - start_size: 100
///  - name: derived
///    start_time: 50
///    ancestors: [ancestor]
///    epochs:
///     - start_size: 10
/// ";
///
/// let graph = demes::loads(yaml).unwrap();
/// ```
pub fn loads(yaml: &str) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_str(yaml)
}

/// Build a [`Graph`](crate::Graph) from a type implementing
/// [`Read`](std::io::Read).
///
/// # Errors
///
/// Returns [`DemesError`](crate::DemesError) in the event of invalid input.
///
/// # Examples
///
/// ```
/// let yaml = "
/// time_units: generations
/// demes:
///  - name: ancestor
///    epochs:
///     - start_size: 100
///  - name: derived
///    start_time: 50
///    ancestors: [ancestor]
///    epochs:
///     - start_size: 10
/// ";
///
/// // A slice of raw bytes implements std::io::BufReader
/// // which implements Read
/// let raw_bytes: &[u8] = yaml.as_bytes();
///
/// let graph = demes::load(raw_bytes).unwrap();
/// assert_eq!(graph, demes::loads(yaml).unwrap());
///
/// // The more common use case will be to load from a file
///
/// // First, let's create a file
/// // and write our buffer to it.
/// {
///     use std::io::prelude::*;
///     let mut file = std::fs::File::create("model.yaml").unwrap();
///     file.write_all(raw_bytes);
/// }
///
///
/// let mut file = std::fs::File::open("model.yaml").unwrap();
/// let graph_from_file = demes::load(file).unwrap();
/// assert_eq!(graph, graph_from_file);
///
/// // clean up
/// std::fs::remove_file("model.yaml").unwrap();
/// ```
pub fn load<T: Read>(reader: T) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_reader(reader)
}

/// Return the package version given in the
/// `Cargo.toml` file of this crate.
///
/// # Examples
///
/// ```
/// let _ = demes::version();
/// ```
pub fn version() -> &'static str {
    VERSION
}
