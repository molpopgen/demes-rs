use crate::AsymmetricMigration;
use crate::Deme;
use crate::Epoch;
use crate::Graph;
use crate::Pulse;

/// Provides iteration over [`Deme`] instances.
///
/// # FFI use
///
/// * Create with [`crate::ffi::demes_graph_deme_iterator`].
/// * Advance with [`crate::ffi::demes_deme_iterator_next`].
/// * Tear down using [`crate::ffi::demes_deme_iterator_deallocate`]
pub struct DemeIterator {
    graph: std::ptr::NonNull<Graph>,
    deme: usize,
}

impl DemeIterator {
    /// Create a new iterator
    pub fn new(graph: &Graph) -> Self {
        let ptr = graph as *const Graph;
        Self {
            graph: std::ptr::NonNull::new(ptr.cast_mut()).unwrap(),
            deme: 0,
        }
    }
    fn next(&mut self) -> Option<&Deme> {
        self.deme += 1;
        // SAFETY: the pointer is non-null
        unsafe { self.graph.as_ref() }.get_deme(self.deme - 1)
    }
}

impl Iterator for DemeIterator {
    type Item = *const Deme;
    fn next(&mut self) -> Option<Self::Item> {
        self.next().map(|deme| deme as *const Deme)
    }
}

/// Iterator over [`Epoch`].
///
/// # FFI use
///
/// * Create with [`crate::ffi::demes_deme_epoch_iterator`].
/// * Advance with [`crate::ffi::demes_epoch_iterator_next`].
/// * Tear down using [`crate::ffi::demes_epoch_iterator_deallocate`]
pub struct EpochIterator {
    deme: std::ptr::NonNull<Deme>,
    epoch: usize,
}

impl EpochIterator {
    /// Create a new iterator
    pub fn new(deme: &Deme) -> Self {
        let ptr = deme as *const Deme;
        Self {
            deme: std::ptr::NonNull::new(ptr.cast_mut()).unwrap(),
            epoch: 0,
        }
    }
    fn next(&mut self) -> Option<&Epoch> {
        self.epoch += 1;
        // SAFETY: the pointer is non-null
        unsafe { self.deme.as_ref() }.get_epoch(self.epoch - 1)
    }
}

impl Iterator for EpochIterator {
    type Item = *const Epoch;
    fn next(&mut self) -> Option<Self::Item> {
        self.next().map(|epoch| epoch as *const Epoch)
    }
}

/// Iterator over [`AsymmetricMigration`]
/// # FFI use
///
/// * Create with [`crate::ffi::demes_graph_asymmetric_migration_iterator`].
/// * Advance with [`crate::ffi::demes_asymmetric_migration_iterator_next`].
/// * Tear down using [`crate::ffi::demes_asymmetric_migration_iterator_deallocate`]
pub struct AsymmetricMigrationIterator {
    migrations_start: Option<*const AsymmetricMigration>,
    current: isize,
    num_migrations: usize,
}

impl AsymmetricMigrationIterator {
    /// Create a new iterator
    pub fn new(graph: &Graph) -> Self {
        let ptr = if graph.migrations().is_empty() {
            None
        } else {
            Some(graph.migrations().as_ptr())
        };
        Self {
            migrations_start: ptr,
            current: 0,
            num_migrations: graph.migrations().len(),
        }
    }

    fn next(&mut self) -> Option<*const AsymmetricMigration> {
        if let Some(ptr) = self.migrations_start {
            let c = self.current;
            self.current += 1;
            if c < (self.num_migrations as isize) {
                // SAFETY: the pointer is non-null and the bounds are checked
                Some(unsafe { ptr.offset(c) })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Iterator for AsymmetricMigrationIterator {
    type Item = *const AsymmetricMigration;
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// Iterator over [`Pulse`]
/// Iterator over [`AsymmetricMigration`]
/// # FFI use
///
/// * Create with [`crate::ffi::demes_graph_pulse_iterator`].
/// * Advance with [`crate::ffi::demes_pulse_iterator_next`].
/// * Tear down using [`crate::ffi::demes_pulse_iterator_deallocate`]
pub struct PulseIterator {
    pulses_start: Option<*const Pulse>,
    current: isize,
    num_pulses: usize,
}

impl PulseIterator {
    /// Create a new iterator
    pub fn new(graph: &Graph) -> Self {
        let ptr = if graph.pulses().is_empty() {
            None
        } else {
            Some(graph.pulses().as_ptr())
        };
        // SAFETY: ptr is correctly initialized
        // and the lenght of pulses is correct.
        Self {
            pulses_start: ptr,
            num_pulses: graph.pulses().len(),
            current: 0,
        }
    }

    fn next(&mut self) -> Option<*const Pulse> {
        if let Some(ptr) = self.pulses_start {
            assert!(!ptr.is_null());
            let c = self.current;
            self.current += 1;
            if c < (self.num_pulses as isize) {
                // SAFETY: the pointer is non-null and the bounds are checked
                Some(unsafe { ptr.offset(c) })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Iterator for PulseIterator {
    type Item = *const Pulse;
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// Iterator over the ancestors of a deme
/// and the corresponding ancestry proportions.
#[repr(C)]
pub struct DemeAncestor {
    /// The ancestor deme
    pub deme: *const Deme,
    /// The ancestry proportion due to [`DemeAncestor::deme`]
    pub proportion: f64,
}

/// Iterator over [`DemeAncestor`]
pub struct DemeAncestorIterator {
    parent_graph: std::ptr::NonNull<Graph>,
    offspring_deme: std::ptr::NonNull<Deme>,
    num_ancestors: usize,
    current_ancestor: usize,
    output: DemeAncestor,
}

impl DemeAncestorIterator {
    /// Create a new iterator
    ///
    /// # Safety
    ///
    /// * `graph` and `deme` must both be non-NULL pointers
    /// * `graph` must point to the parent object of `deme`
    pub unsafe fn new(graph: *const Graph, deme: *const Deme) -> Self {
        debug_assert!(!deme.is_null());
        debug_assert!(!graph.is_null());
        let offspring_deme = std::ptr::NonNull::new_unchecked(deme.cast_mut());
        let num_ancestors = offspring_deme.as_ref().num_ancestors();
        Self {
            parent_graph: std::ptr::NonNull::new_unchecked(graph.cast_mut()),
            offspring_deme,
            num_ancestors,
            current_ancestor: 0,
            output: DemeAncestor {
                deme: std::ptr::null(),
                proportion: f64::NAN,
            },
        }
    }

    // Calls to this are only safe if the safety invariants of DemeAncestorIterator::new
    // are upheld.
    unsafe fn next(&mut self) -> Option<*const DemeAncestor> {
        let c = self.current_ancestor;
        self.current_ancestor += 1;

        if c < self.num_ancestors {
            let deme = &self.parent_graph.as_ref().demes()
                [self.offspring_deme.as_ref().ancestor_indexes()[c]];
            let proportion: f64 = self.offspring_deme.as_ref().proportions()[c].into();
            self.output.deme = deme;
            self.output.proportion = proportion;
            Some(&self.output)
        } else {
            None
        }
    }
}

impl Iterator for DemeAncestorIterator {
    type Item = *const DemeAncestor;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: this is ONLY safe if the invariants of DemeAncestorIterator::new
        // are upheld
        unsafe { self.next() }
    }
}
