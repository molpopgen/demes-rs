mod macros;

mod error;
pub mod specification;

use std::io::Read;

pub use error::DemesError;

pub fn loads(yaml: &str) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_str(yaml)
}

pub fn load<T: Read>(reader: T) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_reader(reader)
}
