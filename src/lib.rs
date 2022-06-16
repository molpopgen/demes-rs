//! rust support for
//! [demes](https://popsim-consortium.github.io/demes-spec-docs).
//!
//! # Example of YAML input
//!
//! ```
//!     let yaml = "
//! description: The Gutenkunst et al. (2009) OOA model.
//! doi:
//! - https://doi.org/10.1371/journal.pgen.1000695
//! time_units: years
//! generation_time: 25
//!
//! demes:
//! - name: ancestral
//!   description: Equilibrium/root population
//!   epochs:
//!   - {end_time: 220e3, start_size: 7300}
//! - name: AMH
//!   description: Anatomically modern humans
//!   ancestors: [ancestral]
//!   epochs:
//!   - {end_time: 140e3, start_size: 12300}
//! - name: OOA
//!   description: Bottleneck out-of-Africa population
//!   ancestors: [AMH]
//!   epochs:
//!   - {end_time: 21.2e3, start_size: 2100}
//! - name: YRI
//!   description: Yoruba in Ibadan, Nigeria
//!   ancestors: [AMH]
//!   epochs:
//!   - start_size: 12300
//! - name: CEU
//!   description: Utah Residents (CEPH) with Northern and Western European Ancestry
//!   ancestors: [OOA]
//!   epochs:
//!   - {start_size: 1000, end_size: 29725}
//! - name: CHB
//!   description: Han Chinese in Beijing, China
//!   ancestors: [OOA]
//!   epochs:
//!   - {start_size: 510, end_size: 54090}
//!
//! migrations:
//! - {demes: [YRI, OOA], rate: 25e-5}
//! - {demes: [YRI, CEU], rate: 3e-5}
//! - {demes: [YRI, CHB], rate: 1.9e-5}
//! - {demes: [CEU, CHB], rate: 9.6e-5}
//! ";
//!
//! let graph = match demes::loads(yaml) {
//!     Ok(graph) => graph,
//!     Err(e) => panic!("{}", e),
//! };
//!
//! {
//!     // round trip back into yaml
//!     let yaml_from_graph = serde_yaml::to_string(&graph).unwrap();
//!     let roundtripped_graph = demes::loads(&yaml_from_graph).unwrap();
//!     assert_eq!(graph, roundtripped_graph);
//! }
//!
//!
//!for deme in graph.demes() {
//!    println!("{} {} {} {} {}",
//!         deme.name(),
//!         deme.start_time(),
//!         deme.end_time(),
//!         deme.start_size(),
//!         deme.end_size());
//!     // A HashMap maps ancestor name -> ancestor Deme
//!     for ancestor in deme.ancestors().keys() {
//!         println!("{} is an ancestor of {}", ancestor, deme.name());
//!     }
//!     // Ref<'_, [String]> of ancestor names
//!     for ancestor in deme.ancestor_names().iter() {
//!         println!("{} is an ancestor of {}", ancestor, deme.name());
//!     }
//!}
//!
//!for m in graph.migrations() {
//!    println!("{} -> {} at rate {} from: {}, to: {}",
//!        m.source(),
//!        m.dest(),
//!        m.rate(),
//!        m.start_time(),
//!        m.end_time());
//!}
//! ```

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
