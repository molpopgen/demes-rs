//! # Forward-time traversal of demes models.
//!
//! ## Re-exports
//!
//! This crate re-exports `demes`.
//! Client code does not have to list `demes`
//! as a cargo dependency, guaranteeing that
//! a compatible version is available.
//!
//! ```{rust}
//! use demes_forward::demes;
//!
//! let yaml = "
//! time_units: generations
//! demes:
//!  - name: a_deme
//!    epochs:
//!     - start_size: 100
//! ";
//! assert!(demes::loads(yaml).is_ok());
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

mod current_size;
mod error;
mod graph;
mod iterators;
mod square_matrix;
mod time;

pub use current_size::CurrentSize;
pub use demes;
pub use error::DemesForwardError;
pub use graph::ForwardGraph;
pub use iterators::StateIteratorDuration;
pub use time::ForwardTime;

/// The size of a deme at a given time.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct DemeSizeAt {
    time: demes::Time,
    forward_time: ForwardTime,
    size: CurrentSize,
}

impl DemeSizeAt {
    /// The current time (in the past)
    pub fn time(&self) -> demes::Time {
        self.time
    }
    /// The current time measured since the
    /// start of the model, forwards in time
    pub fn forward_time(&self) -> ForwardTime {
        self.forward_time
    }
    /// The current size
    pub fn size(&self) -> CurrentSize {
        self.size
    }
}
