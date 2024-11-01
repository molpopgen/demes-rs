//! Define a foreign function interface (FFI) for this crate.
//!
//! The FFI allows demes to be used by any language compatible
//! with the C calling convention.
//!
//! We recommend [cbindgen](https://crates.io/crates/cbindgen) to
//! generate a C or C++ header file from this module.
//!
//! We suggest [corrosion](https://github.com/corrosion-rs/corrosion)
//! for building using `cmake`.
//!
//! It should be possible to use other build systems such as `meson`.
//! However, we have not experimented with this.
//! We will update this section if and when we do so.
//!
//! See the `c_example` folder in the `demes` folder of this crate's
//! [repository](https://github.com/molpopgen/demes-rs) for a fully
//! worked out example.
//!
//! # Notes
//!
//! The rust API stores all strings as [`String`], which is very different
//! from the C pointer to [`std::ffi::c_char`].
//! Therefore, most functions returning `* c_char` return a *copy* of the
//! data stored by rust.
//! It is up to the client code to free these returned data.
//!
//! Functions returning pointers all document if the return value must be
//! freed and, if so, how to do so.
//!
//! Many of the function in this module do not have an `unsafe` label.
//! These labels are correct.
//! When called from rust, these functions are indeed safe.
//! The borrow checker prevents them from being unsafe.
//!
//! However, when called from languages like `C`, this API is subject
//! to the same safety pitfalls as any API for that language.
//! Witout rust's borrow checker, it is up to client code to make
//! sure that parent objects ([`Graph`]s for example) are still valid
//! when child objects ([`Deme`]s for example) are passed to API functions.

use std::ffi::{CStr, CString};

pub use crate::ffi_iterators::AsymmetricMigrationIterator;
pub use crate::ffi_iterators::DemeAncestor;
pub use crate::ffi_iterators::DemeAncestorIterator;
pub use crate::ffi_iterators::DemeIterator;
pub use crate::ffi_iterators::EpochIterator;
pub use crate::ffi_iterators::PulseIterator;
use crate::AsymmetricMigration;
use crate::Deme;
use crate::Epoch;
use crate::Graph;
use crate::Pulse;
use std::os::raw::{c_char, c_int};

enum ErrorDetails {
    UnexpectedNullPointer,
    BoxedError(Box<dyn std::error::Error>),
}

impl std::fmt::Debug for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorDetails::UnexpectedNullPointer => write!(f, "unexpected null pointer"),
            ErrorDetails::BoxedError(e) => write!(f, "{e:?}"),
        }
    }
}

/// Opaque error type.
///
/// This type will usually be used via C.
/// See the `c_example` folder of this crate's repository.
#[derive(Default, Debug)]
pub struct FFIError {
    error: Option<ErrorDetails>,
}

/// The size function for an epoch.
///
/// This enum is a mapping of [`demes::SizeFunction`](crate::SizeFunction)
/// to a representation that can be understood by `C`.
#[repr(C)]
pub enum SizeFunction {
    /// No size change
    Constant,
    /// Exponential size change
    Exponential,
    /// Linear size change
    Linear,
}

/// Allocate a [`FFIError`]
///
/// # Notes
///
/// The memory for this type is allocated via the rust allocator.
/// Therefore, instances must be freed using [`demes_error_deallocate`].
#[no_mangle]
pub extern "C" fn demes_error_allocate() -> *mut FFIError {
    let error = Box::new(FFIError { error: None });
    Box::leak(error)
}

/// Check if a [`FFIError`] contains an error state.
///
/// # Returns
///
/// `true` if there is an error and `false` otherwise.
#[no_mangle]
pub extern "C" fn demes_error_has_error(error: &FFIError) -> bool {
    error.error.is_some()
}

/// Clear error state.
#[no_mangle]
pub extern "C" fn demes_error_clear(error: &mut FFIError) {
    error.error = None
}

/// Obtain a C string containing the error message.
///
/// # Returns
///
/// * The error message, if one exists .
///   The allocated memory **must** be freed via the [`demes_c_char_deallocate`] function,
///   else a memory leak will occur.
/// * A NULL pointer if there is no error.
#[no_mangle]
pub extern "C" fn demes_error_message(error: &FFIError) -> *mut c_char {
    match &error.error {
        None => std::ptr::null_mut(),
        Some(e) => {
            let msg = format!("{:?}", e);
            str_to_owned_c_char(&msg)
        }
    }
}

/// Free the memory for a [`FFIError`]
///
/// # Safety
///
/// * `error` must point to a non-NULL instance of [`FFIError`]
/// * This function must be called at most once on any instance.
#[no_mangle]
pub unsafe extern "C" fn demes_error_deallocate(error: *mut FFIError) {
    assert!(!error.is_null());
    // SAFETY: we have checked that it is not NULL and we are not doing a "double free""
    let _ = Box::from_raw(error);
}

/// Free the memory for a C-style string that was allocated by this module.
///
/// # Safety
///
/// * `ptr` must not be NULL.
/// * `ptr` must point to a [`c_char`].
/// * `ptr` must have been allocated by a function in this crate.
/// * This function must be called at most once on a single allocation.
/// * The input value must satisfy the safety criteria
///   of [`CString::from_raw`].
#[no_mangle]
pub unsafe extern "C" fn demes_c_char_deallocate(ptr: *mut c_char) {
    let _ = CString::from_raw(ptr);
}

fn str_to_owned_c_char(string: &str) -> *mut c_char {
    if string.is_empty() {
        return std::ptr::null_mut();
    }
    // Why do we allow a panic here?
    // This function starts with a RUST string that
    // has been created by other parts of the API.
    // Therefore, we are VERY unlikely to have nul
    // bytes anywhere in the input.
    CString::new(string)
        .expect("String must not contain nul bytes")
        .into_raw()
}

unsafe fn loads(yaml: &str, error: &mut FFIError, output: *mut *mut Graph) -> c_int {
    assert!(error.error.is_none());
    match crate::loads(yaml) {
        Ok(g) => {
            let graph = Box::new(g);
            *output = Box::leak(graph);
            0
        }
        Err(e) => {
            // NOTE: this is why fn is unsafe
            error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
            *output = std::ptr::null_mut();
            1
        }
    }
}

unsafe fn load(file: std::fs::File, error: &mut FFIError, output: *mut *mut Graph) -> c_int {
    assert!(error.error.is_none());
    match crate::load(file) {
        Ok(g) => {
            let graph = Box::new(g);
            // NOTE: this is why fn is unsafe
            *output = Box::leak(graph);
            0
        }
        Err(e) => {
            error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
            *output = std::ptr::null_mut();
            1
        }
    }
}

/// Generate a copy of a [`Graph`] with time units changed to generations.
///
/// # Returns
///
/// * 0 upon success
/// * non-zero upon error
///
/// # Safety
///
/// * `output` must point to a mutable pointer to a [`Graph`]
///
/// # Side effects
///
/// * `output` is overwritten to point to the modified graph upon success.
/// * `output` is overwritten with a NULL pointer upon failure.
///
/// # Errors
///
/// If the time unit of an event differs sufficiently in
/// magnitude from the `generation_time`, it is possible
/// that conversion results in epochs (or migration
/// durations) of length zero, which will return an error.
///
/// If any field is unresolved, an error will be returned.
#[no_mangle]
pub unsafe extern "C" fn demes_graph_into_generations(
    graph: &Graph,
    error: &mut FFIError,
    output: *mut *mut Graph,
) -> c_int {
    match graph.clone().into_generations() {
        Ok(graph_in_generations) => {
            *output = Box::leak(Box::new(graph_in_generations));
            0
        }
        Err(e) => {
            error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
            *output = std::ptr::null_mut();
            1
        }
    }
}

/// Generate a copy of a [`Graph`] with all [`Epoch`] start/end
/// times rounded to the nearest integer values.
///
/// # Returns
///
/// * 0 upon success
/// * non-zero upon error
///
/// # Safety
///
/// * `output` must point to a mutable pointer to a [`Graph`]
///
/// # Side effects
///
/// * `output` is overwritten to point to the modified graph upon success.
/// * `output` is overwritten with a NULL pointer upon failure.
///
/// # Errors
///
/// It is possible that rounding result in invalid epoch lengths,
/// migrations that fall outside of a valid time interval, etc.,
/// which will trigger an error.
#[no_mangle]
pub unsafe extern "C" fn demes_graph_into_integer_generations(
    graph: &Graph,
    error: &mut FFIError,
    output: *mut *mut Graph,
) -> c_int {
    match graph.clone().into_integer_generations() {
        Ok(graph_in_generations) => {
            *output = Box::leak(Box::new(graph_in_generations));
            0
        }
        Err(e) => {
            error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
            *output = std::ptr::null_mut();
            1
        }
    }
}

/// Generate a copy of a [`Graph`] with all [`Epoch`] start/end
/// sizes rounded to the nearest integer values.
///
/// # Returns
///
/// * 0 upon success
/// * non-zero upon error
///
/// # Safety
///
/// * `output` must point to a mutable pointer to a [`Graph`]
///
/// # Side effects
///
/// * `output` is overwritten to point to the modified graph upon success.
/// * `output` is overwritten with a NULL pointer upon failure.
///
/// # Errors
///
/// It is possible that rounding result in invalid deme sizes,
/// which will trigger an error.
#[no_mangle]
pub unsafe extern "C" fn demes_graph_into_integer_start_end_sizes(
    graph: &Graph,
    error: &mut FFIError,
    output: *mut *mut Graph,
) -> c_int {
    match graph.clone().into_integer_start_end_sizes() {
        Ok(graph_in_generations) => {
            *output = Box::leak(Box::new(graph_in_generations));
            0
        }
        Err(e) => {
            error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
            *output = std::ptr::null_mut();
            1
        }
    }
}

/// Free the memory for a [`Graph`]
///
/// # Safety
///
/// * `graph` must point to a non-null instance of [`Graph`]
/// * This function must be called at most once on any instance.
#[no_mangle]
pub unsafe extern "C" fn demes_graph_deallocate(graph: *mut Graph) {
    let _ = Box::from_raw(graph);
}

/// Initialize a [`Graph`] from a YAML string.
///
/// # Returns
///
/// * An initialized [`Graph`] upon success.
/// * A null pointer upon error
/// * A null pointer if `error` contains an error state,
///   implying either an error has not been handled and/or
///   it has not been cleared.
///
/// # Error
///
/// `error` will be set to contain an error state if any error
/// occurs.
///
/// # Safety
///
/// * `yaml` must be a non-null `char *`.
/// * For `yaml`, all safety requirements of
///   [`CStr::from_ptr`](std::ffi::CStr::from_ptr)
///   must be upheld.
/// * `error` must be a non-null pointer to a [`FFIError`]
///
/// # Note
///
/// The return value, if not NULL, **must** be freed via
/// [`demes_graph_deallocate`], else a memory leak will occur.
#[no_mangle]
pub unsafe extern "C" fn demes_graph_load_from_yaml(
    // NOTE: it is very hard to test invalid c style strings.
    yaml: *const c_char,
    error: &mut FFIError,
    output: *mut *mut Graph,
) -> c_int {
    assert!(error.error.is_none());

    if yaml.is_null() {
        error.error = Some(ErrorDetails::UnexpectedNullPointer);
        return 1;
    }

    // WARNING: from_ptr has a LOT in its SAFETY section!
    let yaml = CStr::from_ptr(yaml);
    match yaml.to_owned().to_str() {
        Ok(s) => loads(s, error, output),
        Err(e) => {
            error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
            *output = std::ptr::null_mut();
            1
        }
    }
}

/// Initialize a [`Graph`] from a file.
///
/// # Returns
///
/// * An initialized [`Graph`] upon success.
/// * A null pointer upon error
/// * A null pointer if `error` contains an error state,
///   implying either an error has not been handled and/or
///   it has not been cleared.
///
/// # Error
///
/// `error` will be set to contain an error state if any error
/// occurs.
///
/// # Safety
///
/// * `filename` must be a non-null pointer.
/// * For `filename`, all safety requirements of
///   [`CStr::from_ptr`](std::ffi::CStr::from_ptr)
///   must be upheld.
/// * `error` must be a non-null pointer to a [`FFIError`]
///
/// # Note
///
/// The return value, if not NULL, **must** be freed via
/// [`demes_graph_deallocate`], else a memory leak will occur.
#[no_mangle]
pub unsafe extern "C" fn demes_graph_load_from_file(
    // NOTE: it is very hard to test invalid c style strings.
    filename: *const c_char,
    error: &mut FFIError,
    output: *mut *mut Graph,
) -> c_int {
    assert!(error.error.is_none());
    if filename.is_null() {
        // There is no input string, so fill error
        error.error = Some(ErrorDetails::UnexpectedNullPointer);
        return 1;
    }

    // WARNING: from_ptr has a LOT in its SAFETY section!
    let filename = CStr::from_ptr(filename);
    match filename.to_str() {
        Ok(s) => match std::fs::File::open(s) {
            Ok(file) => load(file, error, output),
            Err(e) => {
                error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
                *output = std::ptr::null_mut();
                1
            }
        },
        Err(e) => {
            error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
            *output = std::ptr::null_mut();
            1
        }
    }
}

/// Get the number of demes in a [`Graph`]
#[no_mangle]
pub extern "C" fn demes_graph_num_demes(graph: &Graph) -> usize {
    graph.num_demes()
}

/// Get a pointer to a [`Deme`] from a [`Graph`]
#[no_mangle]
pub extern "C" fn demes_graph_deme(graph: &Graph, at: usize) -> *const Deme {
    match graph.demes().get(at) {
        Some(deme) => deme,
        None => std::ptr::null(),
    }
}

/// Get a pointer to a [`Deme`] from a [`Graph`] using a deme name
///
/// # Returns
///
/// A non-null pointer to a [`Deme`] if `name` is a valid name in the graph.
/// If `name` is not the name of a deme in the graph, a NULL pointer is returned.
///
/// # Safety
///
/// * `name` must be non-NULL, nul-terminated string
#[no_mangle]
pub unsafe extern "C" fn demes_graph_deme_from_name(
    graph: &Graph,
    name: *const c_char,
) -> *const Deme {
    let n = unsafe { CStr::from_ptr(name) }.to_str().unwrap();
    match graph.get_deme(n) {
        Some(deme) => deme,
        None => std::ptr::null(),
    }
}

/// Return a string representation of the [`Graph`]
///
/// # Returns
///
/// * 0 upon success
/// * Non-zero otherwise
///
/// # Side effects
///
/// * Upon success, the input pointee (`*output`) will be overwritten
///   with a string in YAML format.
/// * Upon error, the output pointee will be overwritten with a NULL
///   pointer.
///
/// # Safety
///
/// * `output` must be a non-NULL pointer to a pointer.
///
/// # Notes
///
/// * The output value pointee, if not NULL, **must** be freed by [`demes_c_char_deallocate`]
#[no_mangle]
pub unsafe extern "C" fn demes_graph_to_yaml(
    graph: &Graph,
    error: &mut FFIError,
    output: *mut *mut c_char,
) -> c_int {
    assert!(error.error.is_none());
    match serde_yaml::to_string(graph) {
        Ok(yaml) => {
            *output = str_to_owned_c_char(&yaml);
            0
        }
        Err(e) => {
            error.error = Some(ErrorDetails::BoxedError(e.into()));
            1
        }
    }
}

/// Return a string representation of the [`Graph`]'s toplevel metadata
/// in YAML format
///
/// # Returns
///
/// * 0 upon success
/// * non-zero upon error
///
/// # Side effects
///
/// * Upon success, the input pointee (`*output`) will be overwritten
///   with a string in YAML format.
/// * Upon error, the output pointee will be overwritten with a NULL
///   pointer.
///
/// # Safety
///
/// * `output` must be a non-NULL pointer to a pointer.
///
/// # Notes
///
/// * The output value pointee, if not NULL, **must** be freed by [`demes_c_char_deallocate`].
#[no_mangle]
pub unsafe extern "C" fn demes_graph_toplevel_metadata_yaml(
    graph: &Graph,
    error: &mut FFIError,
    output: *mut *mut c_char,
) -> c_int {
    assert!(error.error.is_none());
    match graph.metadata() {
        None => {
            *output = std::ptr::null_mut();
            0
        }
        Some(metadata) => match metadata.as_yaml_string() {
            Err(e) => {
                error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
                1
            }
            Ok(metadata) => {
                *output = str_to_owned_c_char(&metadata);
                0
            }
        },
    }
}

/// Get the number of [`Pulse`] items in a [`Graph`].
#[no_mangle]
pub extern "C" fn demes_graph_num_pulses(graph: &Graph) -> usize {
    graph.pulses().len()
}

/// Get a pointer to an [`Pulse`] from a [`Graph`]
///
/// # Notes
///
/// * The return value is NULL if `at` is out of range.
/// * A non-NULL return value points to memory managed by rust.
/// * The const-ness of the return value should not be cast away.
/// * The valid range for `at` can be deduced using [`demes_graph_num_pulses`].
#[no_mangle]
pub extern "C" fn demes_graph_pulse(graph: &Graph, at: usize) -> *const Pulse {
    match graph.pulses().get(at) {
        Some(pulse) => pulse,
        None => std::ptr::null(),
    }
}

/// Get the number of [`AsymmetricMigration`] items in a [`Graph`].
#[no_mangle]
pub extern "C" fn demes_graph_num_migrations(graph: &Graph) -> usize {
    graph.migrations().len()
}

/// Get a pointer to an [`AsymmetricMigration`] from a [`Graph`]
///
/// # Notes
///
/// * The return value is NULL if `at` is out of range.
/// * A non-NULL return value points to memory managed by rust.
/// * The const-ness of the return value should not be cast away.
/// * The valid range for `at` can be deduced using [`demes_graph_num_migrations`].
#[no_mangle]
pub extern "C" fn demes_graph_migration(graph: &Graph, at: usize) -> *const AsymmetricMigration {
    match graph.migrations().get(at) {
        Some(migration) => migration,
        None => std::ptr::null(),
    }
}

/// Return a string representation of the [`Graph`]'s toplevel metadata
/// in JSON format
///
/// # Returns
///
/// * A nul-terminated c_char upon success
/// * A NULL pointer upon error or if `error` contains an error state
///   or if there are no toplevel metadata
///
/// # Side effects
///
/// * Upon success, the input pointee (`*output`) will be overwritten
///   with a string in JSON format.
/// * Upon error, the output pointee will be overwritten with a NULL
///   pointer.
///
/// # Safety
///
/// * `output` must be a non-NULL pointers to a pointer.
///
/// # Notes
///
/// * The output value pointee, if not NULL, **must** be freed by [`demes_c_char_deallocate`].
#[cfg(feature = "json")]
#[no_mangle]
pub unsafe extern "C" fn demes_graph_toplevel_metadata_json(
    graph: &Graph,
    error: &mut FFIError,
    output: *mut *mut c_char,
) -> c_int {
    assert!(error.error.is_none());
    match graph.metadata() {
        None => {
            *output = std::ptr::null_mut();
            0
        }
        Some(metadata) => match serde_json::to_string(metadata.as_raw_ref()) {
            Err(e) => {
                error.error = Some(ErrorDetails::BoxedError(Box::new(e)));
                1
            }
            Ok(metadata) => {
                *output = str_to_owned_c_char(&metadata);
                0
            }
        },
    }
}

/// Get the number of epochs in a [`Deme`]
#[no_mangle]
pub extern "C" fn demes_deme_num_epochs(deme: &Deme) -> usize {
    deme.num_epochs()
}

/// Get the ancestry proportions of a [`Deme`].
///
/// The proportions are in the same order that ancestors are listed
/// in the parent [`Graph`].
/// For example, the same order as the return value of [`demes_deme_ancestor_indexes`].
///
/// # Note
///
/// * The number of elements can be obtained via [`demes_deme_num_ancestors`].
/// * The return value is NULL if a deme has no ancestors.
/// * A non-NULL return value points to memory managed by rust.
/// * The return value should not have its const-ness cast away.
#[no_mangle]
pub extern "C" fn demes_deme_proportions(deme: &Deme) -> *const f64 {
    if !deme.proportions().is_empty() {
        deme.proportions().as_ptr().cast::<f64>()
    } else {
        std::ptr::null()
    }
}

/// Get the number of ancestors of a [`Deme`].
#[no_mangle]
pub extern "C" fn demes_deme_num_ancestors(deme: &Deme) -> usize {
    deme.num_ancestors()
}

/// Get a pointer to the indexes of all ancestors of a [`Deme`]
///
/// # Notes
///
/// * The number of elements can be obtained via [`demes_deme_num_ancestors`].
/// * The return value is NULL if a deme has no ancestors.
/// * A non-NULL return value points to memory managed by rust.
/// * The return value should not have its const-ness cast away.
#[no_mangle]
pub extern "C" fn demes_deme_ancestor_indexes(deme: &Deme) -> *const usize {
    if !deme.ancestor_indexes().is_empty() {
        deme.ancestor_indexes().as_ptr()
    } else {
        std::ptr::null()
    }
}

/// Get the name of a [`Deme`].
///
/// # Note
///
/// The output value pointee must be free'd by [`demes_c_char_deallocate`]
/// to avoid a memory leak.
#[no_mangle]
pub extern "C" fn demes_deme_name(deme: &Deme) -> *mut c_char {
    str_to_owned_c_char(deme.name())
}

/// Get the start time of a [`Deme`].
#[no_mangle]
pub extern "C" fn demes_deme_start_time(deme: &Deme) -> f64 {
    deme.start_time().into()
}

/// Get the end time of a [`Deme`].
#[no_mangle]
pub extern "C" fn demes_deme_end_time(deme: &Deme) -> f64 {
    deme.end_time().into()
}

/// Get the start size of a [`Deme`].
#[no_mangle]
pub extern "C" fn demes_deme_start_size(deme: &Deme) -> f64 {
    deme.start_size().into()
}

/// Get the end time of a [`Deme`].
#[no_mangle]
pub extern "C" fn demes_deme_end_size(deme: &Deme) -> f64 {
    deme.end_size().into()
}

/// Get the size of a [`Deme`] at a specific time.
///
/// # Returns
///
/// * 0 if no error occurs
/// * non-zero otherwise
///
/// # Side effects
///
/// * If `time` falls within the `deme``s `[start_time, end_time),
///   `output` is overwritten with the size of `deme` at time `time`
/// * If `time` is outside of that interval OR an error occurs, `output`
///   is overwritten with [`f64::NAN`].
///
/// # Errors
///
/// If the internal calculation of the deme size results in an invalid [`crate::DemeSize`],
/// then this function will return a non-zero value.
///
/// # Safety
///
/// `output` must be a non-NULL pointer to a [`f64`].
#[no_mangle]
pub unsafe extern "C" fn demes_deme_size_at(deme: &Deme, time: f64, output: &mut f64) -> c_int {
    match deme.size_at(time) {
        Ok(time) => {
            *output = time.map_or(f64::NAN, |t| t.into());
            0
        }
        Err(_) => {
            *output = f64::NAN;
            1
        }
    }
}

/// # Get a pointer to an [`Epoch`] of a [`Deme`].
///
#[no_mangle]
pub extern "C" fn demes_deme_epoch(deme: &Deme, at: usize) -> *const Epoch {
    match deme.epochs().get(at) {
        Some(epoch) => epoch,
        None => std::ptr::null(),
    }
}

/// Get the start time of an [`Epoch`].
#[no_mangle]
pub extern "C" fn demes_epoch_start_time(epoch: &Epoch) -> f64 {
    epoch.start_time().into()
}

/// Get the end time of an [`Epoch`].
#[no_mangle]
pub extern "C" fn demes_epoch_end_time(epoch: &Epoch) -> f64 {
    epoch.end_time().into()
}

/// Get the start size of an [`Epoch`].
#[no_mangle]
pub extern "C" fn demes_epoch_start_size(epoch: &Epoch) -> f64 {
    epoch.start_size().into()
}

/// Get the end size of an [`Epoch`].
#[no_mangle]
pub extern "C" fn demes_epoch_end_size(epoch: &Epoch) -> f64 {
    epoch.end_size().into()
}

/// Get the size of an [`Epoch`] at a specific time.
///
/// # Returns
///
/// * 0 if no error occurs
/// * non-zero otherwise
///
/// # Side effects
///
/// * If `time` falls within the `epoch``s `[start_time, end_time),
///   `output` is overwritten with the size of `epoch` at time `time`
/// * If `time` is outside of that interval OR an error occurs, `output`
///   is overwritten with [`f64::NAN`].
///
/// # Errors
///
/// If the internal calculation of the epoch size results in an invalid [`crate::DemeSize`],
/// then this function will return a non-zero value.
///
/// # Safety
///
/// `output` must be a non-NULL pointer to a [`f64`].
#[no_mangle]
pub unsafe extern "C" fn demes_epoch_size_at(epoch: &Epoch, time: f64, output: &mut f64) -> c_int {
    match epoch.size_at(time) {
        Ok(t) => {
            *output = t.map_or(f64::NAN, |time| time.into());
            0
        }
        Err(_) => {
            *output = f64::NAN;
            1
        }
    }
}

/// Get the [`SizeFunction`] of an [`Epoch`].
#[no_mangle]
pub extern "C" fn demes_epoch_size_function(epoch: &Epoch) -> SizeFunction {
    match epoch.size_function() {
        crate::SizeFunction::Linear => SizeFunction::Linear,
        crate::SizeFunction::Exponential => SizeFunction::Exponential,
        crate::SizeFunction::Constant => SizeFunction::Constant,
    }
}

/// Get the source deme of a [`AsymmetricMigration`]
#[no_mangle]
pub extern "C" fn demes_asymmetric_migration_source(
    migration: &AsymmetricMigration,
) -> *mut c_char {
    str_to_owned_c_char(migration.source())
}

/// Get the destination deme of a [`AsymmetricMigration`]
#[no_mangle]
pub extern "C" fn demes_asymmetric_migration_dest(migration: &AsymmetricMigration) -> *mut c_char {
    str_to_owned_c_char(migration.dest())
}

/// Get the rate of a [`AsymmetricMigration`]
#[no_mangle]
pub extern "C" fn demes_asymmetric_migration_rate(migration: &AsymmetricMigration) -> f64 {
    migration.rate().into()
}

/// Get the start time of a [`AsymmetricMigration`]
#[no_mangle]
pub extern "C" fn demes_asymmetric_migration_start_time(migration: &AsymmetricMigration) -> f64 {
    migration.start_time().into()
}

/// Get the end time of a [`AsymmetricMigration`]
#[no_mangle]
pub extern "C" fn demes_asymmetric_migration_end_time(migration: &AsymmetricMigration) -> f64 {
    migration.end_time().into()
}

/// Get the number of source demes of a [`Pulse`].
#[no_mangle]
pub extern "C" fn demes_pulse_num_sources(pulse: &Pulse) -> usize {
    pulse.sources().len()
}

/// Get the source deme of a [`Pulse`].
///
/// # Parameters
///
/// * `at` the index of the pulse.
///  
/// # Returns
///
/// * The name of a source deme if `at` is in range.
/// * A NULL pointer otherwise.
///
/// # Notes
///
/// * A non-NULL return value is a new allocation that must be freed by
///   [`demes_c_char_deallocate`].
/// * [`demes_pulse_num_sources`] can be used to get the range of valid
///   values for `at`.
#[no_mangle]
pub extern "C" fn demes_pulse_source(pulse: &Pulse, at: usize) -> *mut c_char {
    match pulse.sources().get(at) {
        Some(source) => str_to_owned_c_char(source),
        None => std::ptr::null_mut(),
    }
}

/// Get the destination deme of a [`Pulse`].
///
/// # Parameters
///
/// * `at` the index of the pulse.
///  
/// # Returns
///
/// * The name of a destination deme if `at` is in range.
/// * A NULL pointer otherwise.
///
/// # Notes
///
/// * A non-NULL return value is a new allocation that must be freed by
///   [`demes_c_char_deallocate`].
/// * [`demes_pulse_num_sources`] can be used to get the range of valid
///   values for `at`.
#[no_mangle]
pub extern "C" fn demes_pulse_dest(pulse: &Pulse) -> *mut c_char {
    str_to_owned_c_char(pulse.dest())
}

/// Get the time of a [`Pulse`].
#[no_mangle]
pub extern "C" fn demes_pulse_time(pulse: &Pulse) -> f64 {
    pulse.time().into()
}

/// Get a pointer to all ancestry proportions for a [`Pulse`].
///
/// # Notes
///
/// * The return value points to memory managed by rust
///   that will be freed when the parent graph is deallocated.
/// * The number of elements in the return value can be obtained
///   from [`demes_pulse_num_sources`].
#[no_mangle]
pub extern "C" fn demes_pulse_proportions(pulse: &Pulse) -> *const f64 {
    assert!(!pulse.proportions().is_empty());
    pulse.proportions().as_ptr().cast::<f64>()
}

/// Allocate a [`DemeIterator`].
///
/// # Notes
///
/// * You must call [`demes_deme_iterator_deallocate`] to return resources
///   to the system and avoid a resource leak.
#[no_mangle]
pub extern "C" fn demes_graph_deme_iterator(graph: &Graph) -> *mut DemeIterator {
    Box::leak(Box::new(DemeIterator::new(graph)))
}

/// Dellocate a [`DemeIterator`].
///
/// # Safety
///
/// * `ptr` must point to a non-NULL instance of [`DemeIterator`]
/// * It is undefined behavior to all this function more than once on the same instance.
#[no_mangle]
pub unsafe extern "C" fn demes_deme_iterator_deallocate(ptr: *mut DemeIterator) {
    let _ = unsafe { Box::from_raw(ptr) };
}

/// Advance a [`DemeIterator`]
///
/// # Return
///
/// * A const pointer to a [`Deme`] if the iterator is still valid.
/// * A NULL pointer when iteration has ended.
#[no_mangle]
pub extern "C" fn demes_deme_iterator_next(deme_iterator: &mut DemeIterator) -> *const Deme {
    deme_iterator.next().unwrap_or(std::ptr::null())
}

/// Allocate and initialize an [`EpochIterator`].
///
/// # Note
///
/// The return value must be freed using [`demes_epoch_iterator_deallocate`]
#[no_mangle]
pub extern "C" fn demes_deme_epoch_iterator(deme: &Deme) -> *mut EpochIterator {
    Box::leak(Box::new(EpochIterator::new(deme)))
}

/// Advance an [`EpochIterator`].
///
/// # Return
///
/// * A const pointer to an [`Epoch`] if the iterator is still valid
/// * A NULL pointer when iteration has ended.
#[no_mangle]
pub extern "C" fn demes_epoch_iterator_next(epoch_iterator: &mut EpochIterator) -> *const Epoch {
    epoch_iterator.next().unwrap_or(std::ptr::null())
}

/// Deallocate an [`EpochIterator`]
///
/// # Safety
///
/// * `ptr` must be non-NULL
/// * This function must be called at most once on a given input pointer.
#[no_mangle]
pub unsafe extern "C" fn demes_epoch_iterator_deallocate(ptr: *mut EpochIterator) {
    let _ = unsafe { Box::from_raw(ptr) };
}

/// Allocate and initialize a [`PulseIterator`].
///
/// # Notes
///
/// * You must call [`demes_pulse_iterator_deallocate`] to return resources
///   to the system and avoid a resource leak.
#[no_mangle]
pub extern "C" fn demes_graph_pulse_iterator(graph: &Graph) -> *mut PulseIterator {
    Box::leak(Box::new(PulseIterator::new(graph)))
}

/// Advance a [`PulseIterator`].
///
/// # Return
///
/// * A const pointer to a [`Pulse`] if the iterator is still valid
/// * A NULL pointer when iteration has ended.
#[no_mangle]
pub extern "C" fn demes_pulse_iterator_next(pulse_iterator: &mut PulseIterator) -> *const Pulse {
    pulse_iterator.next().unwrap_or(std::ptr::null())
}

/// Deallocate an [`PulseIterator`]
///
/// # Safety
///
/// * `ptr` must be non-NULL
/// * This function must be called at most once on a given input pointer.
#[no_mangle]
pub unsafe extern "C" fn demes_pulse_iterator_deallocate(ptr: *mut PulseIterator) {
    let _ = unsafe { Box::from_raw(ptr) };
}

/// Allocate and initialize an [`AsymmetricMigrationIterator`].
///
/// # Notes
///
/// * You must call [`demes_asymmetric_migration_iterator_deallocate`] to return resources
///   to the system and avoid a resource leak.
#[no_mangle]
pub extern "C" fn demes_graph_asymmetric_migration_iterator(
    graph: &Graph,
) -> *mut AsymmetricMigrationIterator {
    Box::leak(Box::new(AsymmetricMigrationIterator::new(graph)))
}

/// Advance an [`AsymmetricMigrationIterator`].
///
/// # Return
///
/// * A const pointer to an [`AsymmetricMigration`] if the iterator is still valid
/// * A NULL pointer when iteration has ended.
#[no_mangle]
pub extern "C" fn demes_asymmetric_migration_iterator_next(
    asymmetric_migration_iterator: &mut AsymmetricMigrationIterator,
) -> *const AsymmetricMigration {
    asymmetric_migration_iterator
        .next()
        .unwrap_or(std::ptr::null())
}

/// Deallocate an [`AsymmetricMigrationIterator`]
///
/// # Safety
///
/// * `ptr` must be non-NULL
/// * This function must be called at most once on a given input pointer.
#[no_mangle]
pub unsafe extern "C" fn demes_asymmetric_migration_iterator_deallocate(
    ptr: *mut AsymmetricMigrationIterator,
) {
    let _ = unsafe { Box::from_raw(ptr) };
}

/// Allocate and initialize a [`DemeAncestorIterator`].
///
/// This function is `unsafe` because we cannot use liftimes and references
/// to express the required parent/child object relationships and remain
/// FFI compatible.
///
/// # Parameters
///
/// * `deme` - the [`Deme`] whose ancestors we want to iterate over
/// * `graph` - the parent [`Graph`] of `deme`
///
/// # Safety
///
/// * `deme` must not be NULL
/// * `graph` must not be NULL
/// * `graph` must point to the parent object of `deme`
///
/// # Notes
///
/// * You must call [`demes_deme_ancestor_iterator_deallocate`] to return resources
///   to the system and avoid a resource leak.
#[no_mangle]
pub unsafe extern "C" fn demes_deme_ancestor_iterator(
    deme: *const Deme,
    graph: *const Graph,
) -> *mut DemeAncestorIterator {
    Box::leak(Box::new(DemeAncestorIterator::new(graph, deme)))
}

/// Advance a [`DemeAncestorIterator`].
///
/// # Return
///
/// * A const pointer to a [`DemeAncestor`] if the iterator is still valid
/// * A NULL pointer when iteration has ended.
#[no_mangle]
pub extern "C" fn demes_deme_ancestor_iterator_next(
    iterator: &mut DemeAncestorIterator,
) -> *const DemeAncestor {
    iterator.next().unwrap_or(std::ptr::null())
}

/// Deallocate a [`DemeAncestorIterator`]
///
/// # Safety
///
/// * `ptr` must be non-NULL
/// * This function must be called at most once on a given input pointer.
#[no_mangle]
pub unsafe extern "C" fn demes_deme_ancestor_iterator_deallocate(ptr: *mut DemeAncestorIterator) {
    let _ = Box::from_raw(ptr);
}

#[cfg(test)]
fn basic_valid_graph_yaml() -> &'static str {
    "
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
   "
}

#[cfg(test)]
fn basic_valid_graph() -> Graph {
    let yaml = basic_valid_graph_yaml();
    crate::loads(yaml).unwrap()
}

#[cfg(test)]
#[derive(serde::Deserialize)]
struct MyMetadata {
    #[serde(rename = "X")]
    x: i32,
    #[serde(rename = "Y")]
    y: String,
}

#[test]
fn test_deallocate_graph() {
    let graph = Box::leak(Box::new(basic_valid_graph()));
    unsafe { demes_graph_deallocate(graph) };
}

#[test]
fn test_allocate_deallocate_error() {
    let error = demes_error_allocate();
    unsafe { error.as_mut() }.unwrap().error = Some(ErrorDetails::UnexpectedNullPointer);
    unsafe { demes_error_deallocate(error) };
}

#[test]
fn test_graph_loads_from_yaml() {
    let yaml = basic_valid_graph_yaml();
    let yaml = str_to_owned_c_char(yaml);
    let mut graph: *mut Graph = std::ptr::null_mut();
    let mut error = FFIError::default();
    assert_eq!(
        unsafe { demes_graph_load_from_yaml(yaml, &mut error, &mut graph) },
        0
    );
    unsafe { demes_graph_deallocate(graph) };
    unsafe { demes_c_char_deallocate(yaml) };
}

#[test]
fn test_error_clear() {
    let mut error = FFIError {
        error: Some(ErrorDetails::UnexpectedNullPointer),
    };
    demes_error_clear(&mut error);
    assert!(error.error.is_none());
}

#[test]
fn test_error_no_error_message() {
    let error = FFIError::default();
    let m = demes_error_message(&error);
    assert!(m.is_null());
}

#[test]
fn test_error_message() {
    let error = FFIError {
        error: Some(ErrorDetails::UnexpectedNullPointer),
    };
    let m = demes_error_message(&error);
    assert!(!m.is_null());
    let _ = unsafe { CStr::from_ptr(m) }.to_str().unwrap();
    unsafe { demes_c_char_deallocate(m) };
}

#[test]
fn test_miri_str_to_owned_c_char() {
    let s = "unicorns";
    let c = str_to_owned_c_char(s);
    let roundtrip = unsafe { CStr::from_ptr(c) }
        .to_owned()
        .to_str()
        .unwrap()
        .to_owned();
    assert_eq!(roundtrip, s);
    unsafe { demes_c_char_deallocate(c) };
}

#[test]
fn test_miri_str_to_owned_c_char_empty() {
    let s = "";
    let c = str_to_owned_c_char(s);
    assert!(c.is_null());
}

#[test]
fn test_basic_graph_num_demes() {
    let graph = basic_valid_graph();
    let mut output: *mut Graph = std::ptr::null_mut();
    let mut error = FFIError::default();
    assert_eq!(
        unsafe { demes_graph_into_generations(&graph, &mut error, &mut output) },
        0
    );
    assert!(!output.is_null());
    unsafe { demes_graph_deallocate(output) };
    assert_eq!(
        unsafe { demes_graph_into_integer_generations(&graph, &mut error, &mut output) },
        0
    );
    assert!(!output.is_null());
    unsafe { demes_graph_deallocate(output) };
    assert_eq!(
        unsafe { demes_graph_into_integer_start_end_sizes(&graph, &mut error, &mut output) },
        0
    );
    assert!(!output.is_null());
    unsafe { demes_graph_deallocate(output) };
}

#[test]
fn test_basic_graph_conversions() {
    let graph = basic_valid_graph();
    assert_eq!(demes_graph_num_demes(&graph), graph.num_demes());
}

#[test]
fn test_loads_from_null_yaml() {
    let mut graph: *mut Graph = std::ptr::null_mut();
    let mut error = FFIError::default();
    assert_eq!(
        unsafe { demes_graph_load_from_yaml(std::ptr::null(), &mut error, &mut graph) },
        1
    );
    if let Some(error) = error.error {
        assert!(matches!(error, ErrorDetails::UnexpectedNullPointer))
    }
}

#[test]
fn test_basic_graph_first_deme_num_epochs() {
    let graph = basic_valid_graph();
    let deme = demes_graph_deme(&graph, 0);
    assert_eq!(demes_deme_num_epochs(unsafe { deme.as_ref() }.unwrap()), 1);
}

#[test]
fn test_basic_graph_deme_times() {
    let graph = basic_valid_graph();
    for (i, gdeme) in graph.demes().iter().enumerate() {
        let deme = unsafe { demes_graph_deme(&graph, i).as_ref() }.unwrap();
        let t = demes_deme_start_time(deme);
        assert_eq!(gdeme.start_time(), t);
        let t = demes_deme_end_time(deme);
        assert_eq!(gdeme.end_time(), t);
        let t = demes_deme_start_size(deme);
        assert_eq!(gdeme.start_size(), t);
        let t = demes_deme_end_size(deme);
        assert_eq!(gdeme.end_size(), t);
    }
}

#[test]
fn test_basic_graph_deme_sizes() {
    let graph = basic_valid_graph();
    for (i, gdeme) in graph.demes().iter().enumerate() {
        let deme = demes_graph_deme(&graph, i);
        let deme_ref = unsafe { deme.as_ref().unwrap() };
        let s = demes_deme_start_size(deme_ref);
        assert_eq!(gdeme.start_size(), s);
        let s = demes_deme_end_size(deme_ref);
        assert_eq!(gdeme.end_size(), s);

        let mut deme_size = f64::NAN;
        let t = demes_deme_start_time(deme_ref);
        let rv = unsafe { demes_deme_size_at(deme_ref, t, &mut deme_size) };
        assert_eq!(rv, 0);
        if !t.is_infinite() {
            assert!(deme_size.is_nan(), "{deme_size}");
        } else {
            assert_eq!(deme_size, demes_deme_start_size(deme_ref))
        }
        let mut deme_size = 0.0;
        let t = demes_deme_end_time(deme_ref);
        let rv = unsafe { demes_deme_size_at(deme_ref, t, &mut deme_size) };
        assert_eq!(rv, 0);
        assert!((deme_size - demes_deme_end_size(deme_ref)).abs() <= 1e-9);
    }
}

#[test]
fn test_basic_graph_epochs() {
    let graph = basic_valid_graph();
    let mut error = FFIError::default();
    for i in 0..graph.demes.len() {
        let deme = demes_graph_deme(&graph, i);
        let deme_ref = unsafe { deme.as_ref().unwrap() };
        for e in 0..demes_deme_num_epochs(deme_ref) {
            let epoch_ptr = demes_deme_epoch(deme_ref, e);
            let epoch = unsafe { epoch_ptr.as_ref() }.unwrap();
            let start_size = demes_epoch_start_size(epoch);
            let end_size = demes_epoch_end_size(epoch);
            let start_time = demes_epoch_start_time(epoch);
            let end_time = demes_epoch_end_time(epoch);
            let _ = demes_epoch_size_function(epoch);

            let mut deme_size = f64::MIN;
            let rv = unsafe { demes_epoch_size_at(epoch, start_time, &mut deme_size) };
            if start_time.is_finite() {
                assert_eq!(rv, 0, "{e}, {start_time}, {start_size} -> {error:?}");
                assert!(deme_size.is_nan(), "{deme_size} {start_time}");
                error.error = None;
            } else {
                assert_eq!(rv, 0, "{e}, {start_time}, {start_size} -> {error:?}");
                assert_eq!(deme_size, start_size);
            }

            deme_size = f64::MIN;
            let rv = unsafe { demes_epoch_size_at(epoch, end_time, &mut deme_size) };
            assert_eq!(rv, 0);
            assert!((deme_size - end_size).abs() <= 1e-9);
        }
    }
}

#[test]
fn test_loads_error() {
    let yaml = "
 time_units: generations
 metadata:
  X: 3
  Y: unicorns
 demes:
  - name: A
    epochs:
     - end_time: dafasfa
       start_size: 100
    ";
    let mut error = FFIError::default();
    let mut graph = std::ptr::null_mut();
    assert!(unsafe { loads(yaml, &mut error, &mut graph) } != 0);
    assert!(graph.is_null());
    assert!(demes_error_has_error(&error));
}

#[test]
fn test_missing_toplevel_metadata() {
    let yaml = "
 time_units: generations
 demes:
  - name: A
    epochs:
     - end_time: 100
       start_size: 100
    ";
    let graph = crate::loads(yaml).unwrap();
    let mut error = FFIError::default();
    let mut metadata_string = std::ptr::null_mut();
    assert_eq!(
        unsafe { demes_graph_toplevel_metadata_yaml(&graph, &mut error, &mut metadata_string,) },
        0
    );
    assert!(metadata_string.is_null());
}

#[test]
fn test_toplevel_metadata() {
    let yaml = "
 time_units: generations
 metadata:
  X: 3
  Y: unicorns
 demes:
  - name: A
    epochs:
     - end_time: 100
       start_size: 100
    ";
    let graph = crate::loads(yaml).unwrap();
    let mut error = FFIError::default();
    let mut metadata_string = std::ptr::null_mut();
    let rv =
        unsafe { demes_graph_toplevel_metadata_yaml(&graph, &mut error, &mut metadata_string) };
    assert_eq!(rv, 0);
    let cstr = unsafe { CStr::from_ptr(metadata_string) };
    assert_eq!(cstr.to_str().unwrap(), "X: 3\nY: unicorns\n");
    let owned = cstr.to_str().unwrap().to_owned();
    let md: MyMetadata = serde_yaml::from_str(&owned).unwrap();
    assert_eq!(md.x, 3);
    assert_eq!(md.y, "unicorns");
    unsafe { demes_c_char_deallocate(metadata_string) };
    #[cfg(feature = "json")]
    {
        let rv =
            unsafe { demes_graph_toplevel_metadata_json(&graph, &mut error, &mut metadata_string) };
        assert_eq!(rv, 0);
        let cstr = unsafe { CStr::from_ptr(metadata_string) };
        let owned = cstr.to_str().unwrap().to_owned();
        let md: MyMetadata = serde_json::from_str(&owned).unwrap();
        assert_eq!(md.x, 3);
        assert_eq!(md.y, "unicorns");
        unsafe { demes_c_char_deallocate(metadata_string) };
    }
}

#[test]
fn test_graph_to_yaml() {
    let yaml = "
 time_units: generations
 metadata:
  X: 3,
  Y: unicorns
 demes:
  - name: A
    epochs:
     - end_time: 100
       start_size: 100
    ";
    let graph = crate::loads(yaml).unwrap();
    let mut error = FFIError::default();
    let mut c_yaml = std::ptr::null_mut();
    let rv = unsafe { demes_graph_to_yaml(&graph, &mut error, &mut c_yaml) };
    assert_eq!(rv, 0);
    let cstr = unsafe { CStr::from_ptr(c_yaml) };
    let graph_from_c_yaml = crate::loads(cstr.to_str().unwrap()).unwrap();
    assert_eq!(graph, graph_from_c_yaml);
    unsafe {
        demes_c_char_deallocate(c_yaml);
    };
}

#[test]
fn test_ancestors_and_proportions() {
    let yaml = "
        time_units: generations
        demes:
         - name: A
           epochs:
            - start_size: 100
              end_time: 10
         - name: B
           epochs:
            - start_size: 100
              end_time: 10
         - name: C
           ancestors: [A, B]
           proportions: [0.25, 0.75]
           start_time: 10
           epochs:
            - start_size: 100
        ";
    let graph = crate::loads(yaml).unwrap();
    unsafe {
        let deme_ptr = demes_graph_deme(&graph, 0);
        let deme = deme_ptr.as_ref().unwrap();
        assert!(demes_deme_ancestor_indexes(deme).is_null());
        assert!(demes_deme_proportions(deme).is_null());
        let deme_ptr = demes_graph_deme(&graph, 2);
        let deme = deme_ptr.as_ref().unwrap();
        let ancestors = demes_deme_ancestor_indexes(deme);
        let proportions = demes_deme_proportions(deme);
        let num_ancestors = demes_deme_num_ancestors(deme);
        for i in 0..num_ancestors {
            let ancestor = demes_graph_deme(&graph, *ancestors.add(i));
            let name = demes_deme_name(ancestor.as_ref().unwrap());
            let cname = CStr::from_ptr(name).to_str().unwrap();
            assert_eq!(deme.ancestor_names()[i], cname);
            assert_eq!(*proportions.add(i), deme.proportions()[i]);
            demes_c_char_deallocate(name);
        }
    }
}

#[test]
fn test_pulses() {
    let graph = basic_valid_graph();
    let num_pulses = demes_graph_num_pulses(&graph);
    for i in 0..num_pulses {
        let pulse = demes_graph_pulse(&graph, i);
        let pref = unsafe { pulse.as_ref() }.unwrap();
        let time = demes_pulse_time(pref);
        assert_eq!(time, graph.pulses()[i].time());
        let proportions = demes_pulse_proportions(pref);
        let num_proportions = demes_pulse_num_sources(pref);
        let propslice = unsafe { std::slice::from_raw_parts(proportions, num_proportions) };
        for (a, b) in propslice.iter().zip(graph.pulses()[i].proportions().iter()) {
            assert_eq!(a, b);
        }
        for j in 0..num_proportions {
            let source = demes_pulse_source(pref, j);
            let source_ref = unsafe { CStr::from_ptr(source) }.to_str().unwrap();
            assert_eq!(graph.pulses()[i].sources()[j], source_ref);
            unsafe { demes_c_char_deallocate(source) };
            let dest = demes_pulse_dest(pref);
            let dest_ref = unsafe { CStr::from_ptr(dest) }.to_str().unwrap();
            assert_eq!(graph.pulses()[i].dest(), dest_ref);
            unsafe { demes_c_char_deallocate(dest) };
        }
    }
}

#[test]
fn test_migrations() {
    let graph = basic_valid_graph();
    let num_migrations = demes_graph_num_migrations(&graph);
    assert_eq!(num_migrations, graph.migrations().len());
    for (i, gmig) in graph.migrations().iter().enumerate() {
        let migration = demes_graph_migration(&graph, i);
        let migref = unsafe { migration.as_ref() }.unwrap();
        let rate = demes_asymmetric_migration_rate(migref);
        assert_eq!(rate, gmig.rate());
        let start_time = demes_asymmetric_migration_start_time(migref);
        assert_eq!(start_time, gmig.start_time());
        let end_time = demes_asymmetric_migration_end_time(migref);
        assert_eq!(end_time, gmig.end_time());
        let source = demes_asymmetric_migration_source(migref);
        assert_eq!(
            unsafe { CStr::from_ptr(source) }.to_str().unwrap(),
            gmig.source()
        );
        unsafe { demes_c_char_deallocate(source) };
        let dest = demes_asymmetric_migration_dest(migref);
        assert_eq!(
            unsafe { CStr::from_ptr(dest) }.to_str().unwrap(),
            gmig.dest(),
        );
        unsafe { demes_c_char_deallocate(dest) };
    }
}

#[test]
fn test_deme_from_name() {
    let graph = basic_valid_graph();
    for deme in graph.demes().iter() {
        let name = demes_deme_name(deme);
        let deme_ptr = unsafe { demes_graph_deme_from_name(&graph, name) };
        assert!(!deme_ptr.is_null());
        let name_from_ptr = demes_deme_name(unsafe { &*deme_ptr });
        let str_from_name = unsafe { CStr::from_ptr(name) }.to_str().unwrap();
        let str_from_name_from_ptr = unsafe { CStr::from_ptr(name_from_ptr) }.to_str().unwrap();
        assert_eq!(str_from_name, str_from_name_from_ptr);
        unsafe { demes_c_char_deallocate(name) };
        unsafe { demes_c_char_deallocate(name_from_ptr) };
    }
}

#[test]
fn test_deme_iterator() {
    let graph = basic_valid_graph();
    let deme_iterator = demes_graph_deme_iterator(&graph);
    assert!(!deme_iterator.is_null());
    let mut deme = demes_deme_iterator_next(unsafe { deme_iterator.as_mut() }.unwrap());
    let mut ndemes = 0;
    while !deme.is_null() {
        assert_eq!(
            unsafe { deme.as_ref() }.unwrap(),
            graph.get_deme(ndemes).unwrap()
        );
        ndemes += 1;
        deme = demes_deme_iterator_next(unsafe { deme_iterator.as_mut() }.unwrap());
    }
    unsafe { demes_deme_iterator_deallocate(deme_iterator) };
    assert_eq!(ndemes, graph.num_demes())
}

#[test]
fn test_epoch_iterator() {
    let graph = basic_valid_graph();
    let deme = &graph.demes()[0];
    let epochs = deme.epochs().to_vec();
    let mut epochs_from_iterator = vec![];
    let epoch_iterator = demes_deme_epoch_iterator(deme);
    assert!(!epoch_iterator.is_null());
    // SAFETY: iterator is not NULL
    let mut epoch = demes_epoch_iterator_next(unsafe { epoch_iterator.as_mut() }.unwrap());
    while !epoch.is_null() {
        // SAFETY: epoch is not NULL
        epochs_from_iterator.push(*unsafe { epoch.as_ref() }.unwrap());
        epoch = demes_epoch_iterator_next(unsafe { epoch_iterator.as_mut() }.unwrap());
    }
    unsafe { demes_epoch_iterator_deallocate(epoch_iterator) };
    assert_eq!(epochs, epochs_from_iterator);
}

#[test]
fn test_pulse_iterator() {
    let graph = basic_valid_graph();
    let pulses = graph.pulses().to_vec();
    assert!(!pulses.is_empty());
    let mut pulses_from_iterator = vec![];
    let p = demes_graph_pulse_iterator(&graph);
    assert!(!p.is_null());
    // SAFETY: iterator is not NULL
    let mut pulse = demes_pulse_iterator_next(unsafe { p.as_mut() }.unwrap());
    while !pulse.is_null() {
        // SAFETY: pulse is not NULL
        pulses_from_iterator.push(unsafe { pulse.as_ref().unwrap().clone() });
        pulse = demes_pulse_iterator_next(unsafe { p.as_mut() }.unwrap());
    }
    assert_eq!(pulses, pulses_from_iterator);
    unsafe { demes_pulse_iterator_deallocate(p) };
}

#[test]
fn test_asymmetric_migration_iterator() {
    let graph = basic_valid_graph();
    let migrations = graph.migrations().to_vec();
    assert!(!migrations.is_empty());
    let mut migrations_from_iter = vec![];
    let iterator = demes_graph_asymmetric_migration_iterator(&graph);
    assert!(!iterator.is_null());
    // SAFETY: iterator is not NULL
    let mut migration =
        demes_asymmetric_migration_iterator_next(unsafe { iterator.as_mut() }.unwrap());
    while !migration.is_null() {
        // SAFETY: migration is not NULL
        migrations_from_iter.push(unsafe { migration.as_ref().unwrap().clone() });
        migration = demes_asymmetric_migration_iterator_next(unsafe { iterator.as_mut() }.unwrap());
    }
    assert_eq!(migrations, migrations_from_iter);
    // SAFETY: only deallocating one, ptr is not NULL
    unsafe { demes_asymmetric_migration_iterator_deallocate(iterator) };
}

#[test]
fn test_deme_ancestry_iterator() {
    let graph = basic_valid_graph();
    for deme in graph.demes() {
        // SAFETY: deme and graph have the same implicit lifetimes and are not NULL
        let iterator = unsafe { demes_deme_ancestor_iterator(deme, &graph) };
        // SAFETY: iterator is not NULL
        let mut deme_ancestors =
            demes_deme_ancestor_iterator_next(unsafe { iterator.as_mut() }.unwrap());
        let mut ancestors = vec![];
        let mut proportions = vec![];
        while !deme_ancestors.is_null() {
            // SAFETY: we know it is not NULL
            assert!(!unsafe { deme_ancestors.as_ref() }.unwrap().deme.is_null());
            // SAFETY: we know it is not NULL
            ancestors.push(
                unsafe { deme_ancestors.as_ref().unwrap().deme.as_ref() }
                    .unwrap()
                    .clone(),
            );
            proportions.push(unsafe { deme_ancestors.as_ref() }.unwrap().proportion);
            // SAFETY: iterator is not NULL
            deme_ancestors =
                demes_deme_ancestor_iterator_next(unsafe { iterator.as_mut() }.unwrap());
        }
        // SAFETY: iterator is not NULL and we only deallocate it once
        unsafe { demes_deme_ancestor_iterator_deallocate(iterator) };
        assert_eq!(deme.proportions(), &proportions);
        let ancestors_from_deme = deme
            .ancestor_indexes()
            .iter()
            .cloned()
            .map(|index| graph.get_deme(index).unwrap())
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(&ancestors, &ancestors_from_deme);
    }
}
