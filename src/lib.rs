mod macros;

mod error;
mod types;

pub use error::DemesError;
pub use types::*;

pub fn loads(yaml: &str) -> Result<Graph, Box<dyn std::error::Error>> {
    Graph::new_resolved_from_str(yaml)
}
