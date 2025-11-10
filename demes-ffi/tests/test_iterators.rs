static GRAPH_YAML: &str = "
 time_units: generations
 metadata:
  X: 3
  Y: unicorns
 demes:
  - name: A
    epochs:
     - end_time: 100
       start_size: 100
  - name: B
    epochs:
     - end_time: 500
       start_size: 100
     - end_size: 200
       size_function: linear
  - name: C
    start_time: 200
    ancestors: [A, B]
    proportions: [0.5, 0.5]
    epochs:
     - end_time: 25
       start_size: 100
     - end_time: 0
       end_size: 250
 pulses:
 - sources: [B]
   dest: C
   time: 50
   proportions: [0.1]
 migrations:
 - demes: [B, C]
   start_time: 49
   rate: 0.025
 - demes: [A, B]
   start_time: 550
   rate: 1e-4
   ";

#[test]
fn test_iterators() {
    let mut graph: *mut demes_ffi::Graph = std::ptr::null_mut();
    let mut error = demes_ffi::FFIError::default();
    let yaml: *mut i8 = std::ffi::CString::new(GRAPH_YAML)
        .expect("String must not contain nul bytes")
        .into_raw();
    assert_eq!(
        unsafe { demes_ffi::demes_graph_load_from_yaml(yaml, &mut error, &mut graph) },
        0
    );

    assert!(!graph.is_null());
    // SAFETY: we know the graph is not null and that it has been
    // properly initialized by the call to demes_graph_load_from_yaml
    // We are creating a unique reference here so that we can avoid
    // further unsafe code below.
    // The next time we do any non-const operations on the Graph will
    // be when we deallocate it.
    let graph_ref: &demes_ffi::Graph = unsafe { graph.as_ref() }.unwrap();

    let deme_iterator: *mut demes_ffi::DemeIterator =
        demes_ffi::demes_graph_deme_iterator(graph_ref);
    assert!(!deme_iterator.is_null());
    let deme_iterator_ref: &mut demes_ffi::DemeIterator =
        unsafe { deme_iterator.as_mut() }.unwrap();

    let mut deme: *const demes_ffi::Deme = demes_ffi::demes_deme_iterator_next(deme_iterator_ref);
    while !deme.is_null() {
        // SAFETY: we know that deme is not NULL
        // and is a valid value from Graph
        let epoch_iterator: *mut demes_ffi::EpochIterator =
            demes_ffi::demes_deme_epoch_iterator(unsafe { deme.as_ref() }.unwrap());
        let epoch_iterator_ref: &mut demes_ffi::EpochIterator =
            unsafe { epoch_iterator.as_mut() }.unwrap();
        assert!(!epoch_iterator.is_null());
        let mut epoch: *const demes_ffi::Epoch =
            demes_ffi::demes_epoch_iterator_next(epoch_iterator_ref);
        while !epoch.is_null() {
            epoch = demes_ffi::demes_epoch_iterator_next(epoch_iterator_ref);
        }
        unsafe { demes_ffi::demes_epoch_iterator_deallocate(epoch_iterator) };
        deme = demes_ffi::demes_deme_iterator_next(deme_iterator_ref);
    }
    unsafe { demes_ffi::demes_deme_iterator_deallocate(deme_iterator) };

    let pulse_iterator: *mut demes_ffi::PulseIterator =
        demes_ffi::demes_graph_pulse_iterator(graph_ref);
    assert!(!pulse_iterator.is_null());
    let pulse_iterator_ref: &mut demes_ffi::PulseIterator =
        unsafe { pulse_iterator.as_mut() }.unwrap();
    let mut pulse: *const demes_ffi::Pulse =
        demes_ffi::demes_pulse_iterator_next(pulse_iterator_ref);
    while !pulse.is_null() {
        pulse = demes_ffi::demes_pulse_iterator_next(pulse_iterator_ref);
    }
    unsafe {
        demes_ffi::demes_pulse_iterator_deallocate(pulse_iterator);
    }

    let migration_iterator: *mut demes_ffi::AsymmetricMigrationIterator =
        demes_ffi::demes_graph_asymmetric_migration_iterator(graph_ref);
    assert!(!migration_iterator.is_null());
    let migration_iterator_ref: &mut demes_ffi::AsymmetricMigrationIterator =
        unsafe { migration_iterator.as_mut() }.unwrap();
    let mut migration: *const demes_ffi::AsymmetricMigration =
        demes_ffi::demes_asymmetric_migration_iterator_next(migration_iterator_ref);
    while !migration.is_null() {
        migration = demes_ffi::demes_asymmetric_migration_iterator_next(migration_iterator_ref);
    }
    unsafe {
        demes_ffi::demes_asymmetric_migration_iterator_deallocate(migration_iterator);
    }

    unsafe { demes_ffi::demes_graph_deallocate(graph) };
    unsafe { demes_ffi::demes_c_char_deallocate(yaml) };
}
