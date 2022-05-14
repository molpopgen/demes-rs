mod macros;

mod error;
pub mod specification;

pub use error::DemesError;

pub fn loads(yaml: &str) -> Result<specification::Graph, DemesError> {
    specification::Graph::new_resolved_from_str(yaml)
}
