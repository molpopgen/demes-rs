//! rust support for
//! [demes](https://popsim-consortium.github.io/demes-spec-docs).
//!
//! # Introduction
//!
//! This crate provides:
//!
//! * Support for reading `YAML` descriptions of `demes` models.
//!   See [`loads`] and [`load`].
//! * Support for building a demes model using `rust` code.
//!   See [`GraphBuilder`].
//!
//! The output of any of these operations is a fully-resolved
//! [`Graph`].
//!
//! ## More information
//!
//! * See [here](https://popsim-consortium.github.io/demes-spec-docs/main/introduction.html#) for
//!   an overview of `demes`.
//!
//! ## Technical details
//!
//! * `YAML` and [`GraphBuilder`] inputs
//!   support the Human Data Model (HDM) described in the
//!   demes
//!   [specification](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html)
//! * A [`Graph`] is fully-resolved according to the Machine
//!   Data Model (MDM) described in the
//!   [specification](https://popsim-consortium.github.io/demes-spec-docs/main/specification.html).
//!   
//! # Features
//!
//! The following [cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
//! are available:
//!
//! * `json`: enables reading/writing a [`Graph`] in JSON format.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

mod macros;

mod builder;
mod cloning_rate;
mod deme_size;
mod error;
mod graph_operations;
mod migration_rate;
mod proportion;
mod selfing_rate;
mod specification;
mod time;

#[cfg(feature = "json")]
mod process_json;

#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "ffi")]
mod ffi_iterators;

use std::io::Read;

pub use builder::{BuilderError, GraphBuilder};
pub use cloning_rate::{CloningRate, InputCloningRate};
pub use deme_size::{DemeSize, InputDemeSize};
pub use error::DemesError;
pub use migration_rate::{InputMigrationRate, MigrationRate};
pub use proportion::{InputProportion, Proportion};
pub use selfing_rate::{InputSelfingRate, SelfingRate};
pub use specification::*;
pub use time::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build a [`Graph`] from an in-memory [`str`].
///
/// # Errors
///
/// Returns [`DemesError`] in the event of invalid input.
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

/// Generate a [`Graph`] from a JSON string.
#[cfg(feature = "json")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
pub fn loads_json(json: &str) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_json_str(json)
}

/// Generate a [`Graph`] from a TOML string.
#[cfg(feature = "toml")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "toml")))]
pub fn loads_toml(toml: &str) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_toml_str(toml)
}

/// Build a [`Graph`] from a type implementing
/// [`std::io::Read`].
///
/// # Errors
///
/// Returns [`DemesError`] in the event of invalid input.
///
/// # Examples
///
/// ```
/// // We can load graphs from in-memory data in YAML format:
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
/// // A slice of raw bytes implements std::io::BufReader
/// // which implements Read
/// let raw_bytes: &[u8] = yaml.as_bytes();
/// let graph = demes::load(raw_bytes).unwrap();
/// # assert_eq!(graph, demes::loads(yaml).unwrap());
/// # // The more common use case will be to load from a file
/// # // First, let's create a file
/// # // and write our buffer to it.
/// # {
/// #     use std::io::prelude::*;
/// #     let mut file = std::fs::File::create("model.yaml").unwrap();
/// #     file.write_all(raw_bytes);
/// # }
/// // We can also read from files:
/// let file = std::fs::File::open("model.yaml").unwrap();
/// let graph_from_file = demes::load(file).unwrap();
/// # assert_eq!(graph, graph_from_file);
/// # // clean up
/// # std::fs::remove_file("model.yaml").unwrap();
/// ```
pub fn load<T: Read>(reader: T) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_reader(reader)
}

#[cfg(feature = "json")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "json")))]
/// Load a [`Graph`] from a JSON reader.
pub fn load_json<T: Read>(reader: T) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_json_reader(reader)
}

#[cfg(feature = "toml")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "toml")))]
/// Load a [`Graph`] from a TOML reader.
pub fn load_toml<T: Read>(reader: T) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_toml_reader(reader)
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
