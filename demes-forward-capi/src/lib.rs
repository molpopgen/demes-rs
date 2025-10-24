use demes_forward::demes;
use libc::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Read;

/// ## Not Send/Sync
///
/// This type is meant to be used in an FFI context.
/// We therefore deny Send/Sync:
///
/// ```compile_fail
/// fn is_send<T: Send>()  {}
///
/// is_send::<demes_forward_capi::OpaqueForwardGraph>();
/// ```
///
/// ```compile_fail
/// fn is_sync<T: Sync>()  {}
///
/// is_send::<demes_forward_capi::OpaqueForwardGraph>();
/// ```
pub struct OpaqueForwardGraph {
    graph: Option<demes_forward::ForwardGraph>,
    error: Option<CString>,
    current_time: Option<f64>,
    // Rust types containing raw pointers
    // do not get blanket Send/Sync impl.
    // We use a ZST here to prevent those impl
    // for this type.
    deny_send_sync: std::marker::PhantomData<*const ()>,
}

#[repr(i32)]
enum ErrorCode {
    // GraphUninitialized = -1,
    GraphIsNull = -2,
}

impl OpaqueForwardGraph {
    fn update(&mut self, graph: Option<demes_forward::ForwardGraph>, error: Option<String>) {
        self.graph = graph;
        self.update_error(error);
    }

    fn update_error(&mut self, error: Option<String>) {
        self.error = error.map(|e| {
            CString::new(
                e.chars()
                    .filter(|c| c.is_ascii() && c != &'"')
                    .collect::<String>(),
            )
            .unwrap()
        });
    }
}

/// Allocate an [`OpaqueForwardGraph`]
///
/// # Panics
///
/// This function will panic if the pointer allocation fails.
///
/// # Safety
///
/// The pointer is returned by leaking a [`Box`].
/// The pointer is managed by rust and is freed by [`forward_graph_deallocate`].
#[no_mangle]
pub extern "C" fn forward_graph_allocate() -> *mut OpaqueForwardGraph {
    Box::into_raw(Box::new(OpaqueForwardGraph {
        graph: None,
        error: None,
        current_time: None,
        deny_send_sync: std::marker::PhantomData,
    }))
}

unsafe fn yaml_to_owned(yaml: *const c_char) -> Option<String> {
    if yaml.is_null() {
        return None;
    }
    let yaml = CStr::from_ptr(yaml);
    match yaml.to_owned().to_str() {
        Ok(s) => Some(s.to_owned()),
        Err(_) => None,
    }
}

unsafe fn process_input_yaml(
    yaml: *const c_char,
    graph: *mut OpaqueForwardGraph,
) -> (i32, Option<demes::Graph>) {
    if graph.is_null() {
        return (ErrorCode::GraphIsNull as i32, None);
    }
    let yaml = match yaml_to_owned(yaml) {
        Some(s) => s,
        None => {
            (*graph).update(None, Some("could not convert c_char to String".to_string()));
            return (-1, None);
        }
    };
    match demes::loads(&yaml) {
        Ok(graph) => (0, Some(graph)),
        Err(e) => {
            (*graph).update(None, Some(format!("{e}")));
            (-1, None)
        }
    }
}

/// Initialize a discrete-time model from a yaml file
///
/// # Safety
///
/// * `yaml` must be a valid pointer containing valid utf8 data.
/// * `graph` must be a valid pointer to OpaqueForwardGraph.
#[no_mangle]
pub unsafe extern "C" fn forward_graph_initialize_from_yaml(
    yaml: *const c_char,
    burnin: f64,
    graph: *mut OpaqueForwardGraph,
) -> i32 {
    let dg = match process_input_yaml(yaml, graph) {
        (x, Some(g)) => {
            assert_eq!(x, 0);
            g
        }
        (y, None) => {
            assert!(y < 0);
            return -1;
        }
    };

    match demes_forward::ForwardGraph::new_discrete_time(dg, burnin) {
        Ok(fgraph) => (*graph).update(Some(fgraph), None),
        Err(e) => (*graph).update(None, Some(format!("{e}"))),
    };
    0
}

/// Initialize and round epoch start/end sizes.
///
/// # Errors
///
/// If any epoch start/end sizes round to zero.
///
/// # Safety
///
/// * `yaml` must be a valid pointer containing valid utf8 data.
/// * `graph` must be a valid pointer to OpaqueForwardGraph.
#[no_mangle]
pub unsafe extern "C" fn forward_graph_initialize_from_yaml_round_epoch_sizes(
    yaml: *const c_char,
    burnin: f64,
    graph: *mut OpaqueForwardGraph,
) -> i32 {
    let dg = match process_input_yaml(yaml, graph) {
        (x, Some(g)) => {
            assert_eq!(x, 0);
            g
        }
        (y, None) => {
            assert!(y < 0);
            return -1;
        }
    };
    let dg = match dg.into_integer_start_end_sizes() {
        Ok(graph) => graph,
        Err(e) => {
            (*graph).update(None, Some(format!("{e}")));
            return -1;
        }
    };
    match demes_forward::ForwardGraph::new_discrete_time(dg, burnin) {
        Ok(fgraph) => (*graph).update(Some(fgraph), None),
        Err(e) => (*graph).update(None, Some(format!("{e}"))),
    };
    0
}

/// # Safety
///
/// * `file_name` must be a non-NULL pointer to valid utf8.
/// * `graph` must be a valid pointer to an [`OpaqueForwardGraph`].
#[no_mangle]
pub unsafe extern "C" fn forward_graph_initialize_from_yaml_file(
    file_name: *const c_char,
    burnin: f64,
    graph: *mut OpaqueForwardGraph,
) -> i32 {
    if graph.is_null() {
        return ErrorCode::GraphIsNull as i32;
    }
    let filename_cstr = CStr::from_ptr(file_name);
    let filename = match filename_cstr.to_str() {
        Ok(string) => string,
        Err(e) => {
            (*graph).update(None, Some(format!("{e}")));
            return -1;
        }
    };
    match std::fs::File::open(filename) {
        Ok(mut file) => {
            let mut buf = String::default();
            match file.read_to_string(&mut buf) {
                Ok(_) => {
                    let cstring = CString::new(buf).unwrap();
                    let ptr = cstring.as_ptr();
                    forward_graph_initialize_from_yaml(ptr, burnin, graph)
                }
                Err(e) => {
                    (*graph).update(None, Some(format!("{e}")));
                    -1
                }
            }
        }
        Err(e) => {
            (*graph).update(None, Some(format!("{e}")));
            -1
        }
    }
}

/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_is_error_state(graph: *const OpaqueForwardGraph) -> bool {
    (*graph).error.is_some()
}

/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_deallocate(graph: *mut OpaqueForwardGraph) {
    let _ = Box::from_raw(graph);
}

/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_get_error_message(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> *const c_char {
    *status = 0;
    if !graph.is_null() {
        match &(*graph).error {
            Some(message) => message.as_ptr(),
            None => std::ptr::null(),
        }
    } else {
        *status = ErrorCode::GraphIsNull as i32;
        std::ptr::null()
    }
}

/// Pointer to first element of selfing rates array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_selfing_rates(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> *const f64 {
    *status = 0;
    if !graph.is_null() {
        match &(*graph).graph {
            Some(graph) => match graph.selfing_rates() {
                Some(slice) => slice.as_ptr() as *const f64,
                None => std::ptr::null(),
            },
            None => {
                *status = -1;
                std::ptr::null()
            }
        }
    } else {
        *status = ErrorCode::GraphIsNull as i32;
        std::ptr::null()
    }
}

/// Pointer to first element of cloning rates array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_cloning_rates(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> *const f64 {
    *status = 0;
    if !graph.is_null() {
        match &(*graph).graph {
            Some(graph) => match graph.cloning_rates() {
                Some(slice) => slice.as_ptr() as *const f64,
                None => std::ptr::null(),
            },
            None => {
                *status = -1;
                std::ptr::null()
            }
        }
    } else {
        *status = ErrorCode::GraphIsNull as i32;
        std::ptr::null()
    }
}

/// Return a pointer to the first element of parental deme size array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_parental_deme_sizes(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> *const f64 {
    *status = 0;
    if !graph.is_null() {
        match &(*graph).graph {
            Some(graph) => match graph.parental_deme_sizes() {
                Some(slice) => slice.as_ptr() as *const f64,
                None => std::ptr::null(),
            },
            None => {
                *status = -1;
                std::ptr::null()
            }
        }
    } else {
        *status = ErrorCode::GraphIsNull as i32;
        std::ptr::null()
    }
}

/// Return a pointer to the first element of offspring deme size array.
///
/// The length of the array is equal to [`forward_graph_number_of_demes`].
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_offspring_deme_sizes(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> *const f64 {
    *status = 0;
    if !graph.is_null() {
        match &(*graph).graph {
            Some(graph) => match graph.offspring_deme_sizes() {
                Some(slice) => slice.as_ptr() as *const f64,
                None => std::ptr::null(),
            },
            None => {
                *status = -1;
                std::ptr::null()
            }
        }
    } else {
        *status = ErrorCode::GraphIsNull as i32;
        std::ptr::null()
    }
}

/// Check if there are any extant offspring demes.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_any_extant_offspring_demes(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> bool {
    *status = 0;
    if !graph.is_null() {
        match &(*graph).graph {
            Some(graph) => graph.any_extant_offspring_demes(),
            None => {
                *status = -1;
                false
            }
        }
    } else {
        *status = ErrorCode::GraphIsNull as i32;
        false
    }
}

/// Check if there are any extant parental demes.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_any_extant_parent_demes(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> bool {
    *status = 0;
    if !graph.is_null() {
        match &(*graph).graph {
            Some(graph) => graph.any_extant_parental_demes(),
            None => {
                *status = -1;
                false
            }
        }
    } else {
        *status = ErrorCode::GraphIsNull as i32;
        false
    }
}

/// Get the total number of demes in the model
///
/// # Returns
///
/// [`isize`] > 0 if the graph is not in an error state.
/// Returns `-1` otherwise.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_number_of_demes(graph: *const OpaqueForwardGraph) -> isize {
    match &(*graph).graph {
        Some(graph) => graph.num_demes_in_model() as isize,
        None => -1,
    }
}

/// Update the model state to a given time.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_update_state(
    time: f64,
    graph: *mut OpaqueForwardGraph,
) -> i32 {
    if !graph.is_null() {
        match &mut (*graph).graph {
            Some(fgraph) => match fgraph.update_state(time) {
                Ok(()) => 0,
                Err(e) => {
                    (*graph).update(None, Some(format!("{e}")));
                    -1
                }
            },
            None => -1,
        }
    } else {
        ErrorCode::GraphIsNull as i32
    }
}

/// Initialize graph to begin iterating over model.
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_initialize_time_iteration(
    graph: *mut OpaqueForwardGraph,
) -> i32 {
    if !graph.is_null() {
        match &mut (*graph).graph {
            Some(fgraph) => {
                match fgraph.last_time_updated() {
                    Some(value) => {
                        (*graph).current_time = Some(value.value() - 1.0);
                    }
                    None => {
                        (*graph).current_time = Some(-1.0);
                    }
                }
                0
            }
            None => -1,
        }
    } else {
        ErrorCode::GraphIsNull as i32
    }
}

/// Iterate to the next time point in the model.
///
/// # Return values:
///
/// * null = done iterating
/// * not null = still iterating
///
/// # Safety
///
/// `graph` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn forward_graph_iterate_time(
    graph: *mut OpaqueForwardGraph,
    status: *mut i32,
) -> *const f64 {
    if graph.is_null() {
        *status = ErrorCode::GraphIsNull as i32;
        return std::ptr::null();
    }
    *status = 0;
    if (*graph).current_time.is_none() {
        *status = -1;
        (*graph).update_error(Some(
            "forward_graph_initialize_time_iteration has not been called".to_string(),
        ));
        return std::ptr::null();
    }
    let tref: &mut f64 = (*graph).current_time.as_mut().unwrap();
    match &mut (*graph).graph {
        Some(fgraph) => {
            if *tref < fgraph.end_time().value() - 1.0 {
                *tref += 1.0;
                &*tref
            } else {
                (*graph).current_time = None;
                std::ptr::null()
            }
        }
        None => {
            *status = -1;
            std::ptr::null()
        }
    }
}

/// # Safety
///
/// `graph` must be a valid pointer to an [`OpaqueForwardGraph`].
/// `status` must be a valid pointer to an `i32`.
#[no_mangle]
pub unsafe extern "C" fn forward_graph_ancestry_proportions(
    offspring_deme: usize,
    status: *mut i32,
    graph: *mut OpaqueForwardGraph,
) -> *const f64 {
    if graph.is_null() {
        *status = ErrorCode::GraphIsNull as i32;
        return std::ptr::null();
    }
    *status = 0;
    if (*graph).error.is_some() {
        *status = -1;
        return std::ptr::null();
    }
    match &(*graph).graph {
        Some(fgraph) => {
            if offspring_deme >= fgraph.num_demes_in_model() {
                *status = -1;
                (*graph).update_error(Some(format!(
                    "offspring deme index {} out of range",
                    offspring_deme
                )));
                std::ptr::null()
            } else {
                match fgraph.ancestry_proportions(offspring_deme) {
                    Some(proportions) => proportions.as_ptr(),
                    None => std::ptr::null(),
                }
            }
        }
        None => {
            *status = -1;
            std::ptr::null()
        }
    }
}

/// Get the model end time.
///
/// The value returned is one generation after the
/// last parental generation.
/// Thus, this value defines a half-open interval
/// during which parental demes exist.
///
/// # Safety
///
/// `graph` must be a valid pointer to an [`OpaqueForwardGraph`].
/// `status` must be a valid pointer to an `i32`.
#[no_mangle]
pub unsafe extern "C" fn forward_graph_model_end_time(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> f64 {
    *status = 0;
    if (*graph).error.is_some() || (*graph).graph.is_none() {
        *status = -1;
        f64::NAN
    } else {
        match &(*graph).graph {
            Some(fgraph) => fgraph.end_time().value(),
            None => {
                *status = -1;
                f64::NAN
            }
        }
    }
}

/// Get the underlying [`demes::Graph`].
///
/// # Returns
///
/// * A new `libc::c_char` representation of the graph upon success.
/// * `std::ptr::null` upon error.
///
/// # Side effects
///
/// * An error will set `status` to -1.
/// * Success will set `status` to 0.
///
/// # Safety
///
/// `graph` must be a valid pointer to an [`OpaqueForwardGraph`].
/// `status` must be a valid pointer to an `i32`.
///
/// # Note
///
/// If not NULL, the return value must be freed in order to avoid
/// leaking memory.
#[no_mangle]
pub unsafe extern "C" fn forward_graph_get_demes_graph(
    graph: *const OpaqueForwardGraph,
    status: *mut i32,
) -> *mut c_char {
    match &(*graph).graph {
        None => std::ptr::null_mut(),
        Some(g) => {
            let demes_graph = match g.demes_graph().as_string() {
                Ok(g) => g,
                Err(_) => {
                    *status = -1;
                    return std::ptr::null_mut();
                }
            };
            let c_str = match CString::new(demes_graph) {
                Ok(c) => c,
                Err(_) => {
                    *status = -1;
                    return std::ptr::null_mut();
                }
            };
            *status = 0;
            libc::strdup(c_str.as_ptr())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::CString, io::Write};

    struct GraphHolder {
        graph: *mut OpaqueForwardGraph,
    }

    impl GraphHolder {
        fn new() -> Self {
            Self {
                graph: forward_graph_allocate(),
            }
        }

        fn as_mut_ptr(&mut self) -> *mut OpaqueForwardGraph {
            self.graph
        }

        fn as_ptr(&mut self) -> *const OpaqueForwardGraph {
            self.graph
        }

        fn init_with_yaml(&mut self, burnin: f64, yaml: &str) -> i32 {
            let yaml_cstr = CString::new(yaml).unwrap();
            let yaml_c_char: *const c_char = yaml_cstr.as_ptr() as *const c_char;
            unsafe { forward_graph_initialize_from_yaml(yaml_c_char, burnin, self.as_mut_ptr()) }
        }

        fn init_with_yaml_round_epoch_sizes(&mut self, burnin: f64, yaml: &str) -> i32 {
            let yaml_cstr = CString::new(yaml).unwrap();
            let yaml_c_char: *const c_char = yaml_cstr.as_ptr() as *const c_char;
            unsafe {
                forward_graph_initialize_from_yaml_round_epoch_sizes(
                    yaml_c_char,
                    burnin,
                    self.as_mut_ptr(),
                )
            }
        }
    }

    impl Drop for GraphHolder {
        fn drop(&mut self) {
            unsafe { forward_graph_deallocate(self.as_mut_ptr()) };
        }
    }

    #[test]
    fn test_invalid_graph() {
        let yaml = "
time_units: generations
demes:
 - name: A
   start_time: 55
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
        let mut graph = GraphHolder::new();
        graph.init_with_yaml(100.0, yaml);
        assert!(unsafe { forward_graph_is_error_state(graph.as_ptr()) });
        let mut status = -1;
        let pstatus: *mut i32 = &mut status;
        let message = unsafe { forward_graph_get_error_message(graph.as_ptr(), pstatus) };
        assert_eq!(status, 0);
        assert!(!message.is_null());
        let rust_message = unsafe { CStr::from_ptr(message) };
        let rust_message: &str = rust_message.to_str().unwrap();
        assert_eq!(
            rust_message,
            "deme error: deme A has finite start time but no ancestors"
        );
    }

    #[test]
    fn test_empty_graph() {
        let yaml = "";
        let mut graph = GraphHolder::new();
        graph.init_with_yaml(100.0, yaml);
        assert!(unsafe { forward_graph_is_error_state(graph.as_ptr()) });
    }

    #[test]
    fn test_null_graph() {
        let yaml: *const c_char = std::ptr::null();
        let graph = forward_graph_allocate();
        unsafe { forward_graph_initialize_from_yaml(yaml, 100.0, graph) };
        assert!(unsafe { forward_graph_is_error_state(graph) });
        unsafe { forward_graph_deallocate(graph) };
    }

    #[test]
    fn number_of_demes_in_model() {
        {
            let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
            {
                let mut graph = GraphHolder::new();
                graph.init_with_yaml(100.0, yaml);
                let num_demes = unsafe { forward_graph_number_of_demes(graph.as_ptr()) };
                assert_eq!(num_demes, 1);
            }

            // Handles the complications of rust str vs char *
            {
                let graph = forward_graph_allocate();
                let cstr = CString::new(yaml).unwrap();
                unsafe { forward_graph_initialize_from_yaml(cstr.as_ptr(), 100., graph) };
                let num_demes = unsafe { forward_graph_number_of_demes(graph) };
                assert_eq!(num_demes, 1);
                unsafe { forward_graph_deallocate(graph) };
            }
        }
    }

    #[test]
    fn iterate_simple_model() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
        let mut graph = GraphHolder::new();
        graph.init_with_yaml(100.0, yaml);
        let mut status = -1;
        assert!(unsafe { forward_graph_selfing_rates(graph.as_ptr(), &mut status) }.is_null());
        assert_eq!(status, 0);
        status = -1;
        assert!(unsafe { forward_graph_cloning_rates(graph.as_ptr(), &mut status) }.is_null());
        assert_eq!(status, 0);
        status = -1;
        assert!(
            unsafe { forward_graph_parental_deme_sizes(graph.as_ptr(), &mut status) }.is_null(),
        );
        assert_eq!(status, 0);
        status = -1;
        assert!(
            unsafe { forward_graph_offspring_deme_sizes(graph.as_ptr(), &mut status) }.is_null(),
        );
        assert_eq!(status, 0);
        status = -1;
        assert!(!unsafe { forward_graph_any_extant_offspring_demes(graph.as_ptr(), &mut status) });
        assert_eq!(status, 0);
        status = -1;
        assert!(!unsafe { forward_graph_any_extant_parent_demes(graph.as_ptr(), &mut status) });
        assert_eq!(status, 0);

        {
            assert_eq!(
                unsafe { forward_graph_initialize_time_iteration(graph.as_mut_ptr()) },
                0,
            );
            let mut ngens = -1_i32;
            let mut ptime: *const f64;
            let mut ancestry_proportions: *const f64;
            let mut times = vec![];
            let mut sizes = vec![100.0; 100];
            sizes.append(&mut vec![200.0; 50]);

            let mut status = -1;
            let pstatus: *mut i32 = &mut status;
            ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), pstatus) };
            assert_eq!(
                unsafe { forward_graph_model_end_time(graph.as_ptr(), pstatus) },
                151.0
            );
            assert_eq!(status, 0);
            while !ptime.is_null() {
                assert_eq!(status, 0);
                ngens += 1;
                unsafe { times.push(*ptime) };
                assert_eq!(
                    unsafe { forward_graph_update_state(*ptime, graph.as_mut_ptr()) },
                    0,
                );
                let mut status = -1;
                if unsafe { forward_graph_any_extant_offspring_demes(graph.as_ptr(), &mut status) }
                {
                    assert_eq!(status, 0);
                    let offspring_deme_sizes =
                        unsafe { forward_graph_offspring_deme_sizes(graph.as_ptr(), &mut status) };
                    assert_eq!(status, 0);
                    assert!(!offspring_deme_sizes.is_null());
                    ancestry_proportions = unsafe {
                        forward_graph_ancestry_proportions(0, &mut status, graph.as_mut_ptr())
                    };
                    assert_eq!(status, 0);
                    let ancestry_proportions =
                        unsafe { std::slice::from_raw_parts(ancestry_proportions, 1) };
                    assert!((ancestry_proportions[0] - 1.0) <= 1e-9);
                    let deme_sizes = unsafe { std::slice::from_raw_parts(offspring_deme_sizes, 1) };
                    assert_eq!(deme_sizes[0], sizes[ngens as usize]);
                } else {
                    status = -1;
                    let offspring_deme_sizes =
                        unsafe { forward_graph_offspring_deme_sizes(graph.as_ptr(), &mut status) };
                    assert_eq!(status, 0);
                    assert!(offspring_deme_sizes.is_null());
                }
                ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
            }
            assert!(ptime.is_null());
            assert_eq!(times.first().unwrap(), &0.0);
            assert_eq!(times.last().unwrap(), &150.0);
            assert_eq!(ngens, 150);
        }

        // Now, start from time of 50
        {
            assert_eq!(
                unsafe { forward_graph_update_state(50.0, graph.as_mut_ptr()) },
                0,
            );
            assert_eq!(
                unsafe { forward_graph_initialize_time_iteration(graph.as_mut_ptr()) },
                0,
            );
            let mut ngens = -1_i32;
            let mut ptime: *const f64;
            let mut times = vec![];
            let mut sizes = vec![100.0; 50];
            sizes.append(&mut vec![200.0; 50]);

            let mut status = -1;
            ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
            while !ptime.is_null() {
                assert_eq!(status, 0);
                ngens += 1;
                unsafe { times.push(*ptime) };
                assert_eq!(
                    unsafe { forward_graph_update_state(*ptime, graph.as_mut_ptr()) },
                    0,
                );
                let mut status = -1;
                if unsafe { forward_graph_any_extant_offspring_demes(graph.as_ptr(), &mut status) }
                {
                    assert_eq!(status, 0);
                    let offspring_deme_sizes =
                        unsafe { forward_graph_offspring_deme_sizes(graph.as_ptr(), &mut status) };
                    assert_eq!(status, 0);
                    assert!(!offspring_deme_sizes.is_null());
                    let deme_sizes = unsafe { std::slice::from_raw_parts(offspring_deme_sizes, 1) };
                    assert_eq!(deme_sizes[0], sizes[ngens as usize]);
                } else {
                    status = -1;
                    let offspring_deme_sizes =
                        unsafe { forward_graph_offspring_deme_sizes(graph.as_ptr(), &mut status) };
                    assert_eq!(status, 0);
                    assert!(offspring_deme_sizes.is_null());
                }
                ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
            }
            assert!(ptime.is_null());
            assert_eq!(times.first().unwrap(), &50.0);
            assert_eq!(times.last().unwrap(), &150.0);
            assert_eq!(ngens, 100);
        }
    }

    #[test]
    fn test_from_yaml_file() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
        {
            let mut file = std::fs::File::create("simple_model.yaml").unwrap();
            file.write_all(yaml.as_bytes()).unwrap();
        }

        let mut graph = GraphHolder::new();

        let filename = "simple_model.yaml";
        let filename_cstring = CString::new(filename).unwrap();
        let filename: *const c_char = filename_cstring.as_ptr() as *const c_char;
        assert_eq!(
            unsafe { forward_graph_initialize_from_yaml_file(filename, 100.0, graph.as_mut_ptr()) },
            0
        );
        let mut status = -1;
        let pstatus: *mut i32 = &mut status;

        assert_eq!(
            unsafe { forward_graph_model_end_time(graph.as_mut_ptr(), pstatus) },
            151.0
        );

        std::fs::remove_file("simple_model.yaml").unwrap();
    }

    #[test]
    fn test_recover_demes_graph() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
        let mut graph = GraphHolder::new();
        assert_eq!(graph.init_with_yaml(0.0, yaml), 0);
        let mut status = 0;
        let demes_graph =
            unsafe { forward_graph_get_demes_graph(graph.as_ptr(), &mut status) } as *const c_char;
        assert!(!demes_graph.is_null());
        let mut new_graph = GraphHolder::new();
        let temp = unsafe { yaml_to_owned(demes_graph) }.unwrap();
        assert_eq!(new_graph.init_with_yaml(0.0, &temp), 0);
        unsafe { libc::free(demes_graph as *mut libc::c_void) };
        assert_eq!(
            unsafe { &(*graph.graph) }
                .graph
                .as_ref()
                .unwrap()
                .demes_graph(),
            unsafe { &(*new_graph.graph) }
                .graph
                .as_ref()
                .unwrap()
                .demes_graph()
        )
    }

    #[test]
    fn test_zero_length_model() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
";
        let mut graph = GraphHolder::new();
        assert_eq!(graph.init_with_yaml(0.0, yaml), 0);
        assert!(!unsafe { forward_graph_is_error_state(graph.as_ptr()) });
        assert_eq!(
            unsafe { forward_graph_initialize_time_iteration(graph.as_mut_ptr()) },
            0,
        );
        let mut ptime: *const f64;
        let mut status: i32 = -1;
        ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
        let mut ngens = 0;
        while !ptime.is_null() {
            assert_eq!(status, 0);
            ngens += 1;
            ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
        }
        // This is a subtle point:
        // 1. We iterate over parent generation 0.
        // 2. We see that there are no children.
        // 3. So no sampling happens.
        assert_eq!(ngens, 1);

        // How to use the API more correctly vis-a-vis the demes spec

        // 1. Reset things
        assert_eq!(
            unsafe { forward_graph_update_state(0.0, graph.as_mut_ptr()) },
            0
        );
        assert_eq!(
            unsafe { forward_graph_initialize_time_iteration(graph.as_mut_ptr()) },
            0,
        );
        ngens = 0;

        // 2. Iterate
        let _ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };

        // Only continue iteration while there are offspring demes.
        while unsafe { forward_graph_any_extant_offspring_demes(graph.as_ptr(), &mut status) } {
            ngens += 1;
            let _ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
            unsafe { forward_graph_update_state(*_ptime, graph.as_mut_ptr()) };
        }
        assert_eq!(ngens, 0);
    }

    #[test]
    fn test_iteration_with_burnin() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
";
        for start_time in [0.0, 5.0, 10.0] {
            let mut graph = GraphHolder::new();
            assert_eq!(graph.init_with_yaml(10.0, yaml), 0);
            assert!(!unsafe { forward_graph_is_error_state(graph.as_ptr()) });
            let mut status: i32 = 0;
            let mut ngens = 0;

            // We must first initialize the internal state
            // to our starting time.
            assert_eq!(
                unsafe { forward_graph_update_state(start_time, graph.as_mut_ptr()) },
                0
            );

            // Cannot call this until AFTER first call to update state
            assert_eq!(
                unsafe { forward_graph_initialize_time_iteration(graph.as_mut_ptr()) },
                0,
            );
            assert_eq!(
                unsafe { forward_graph_model_end_time(graph.as_ptr(), &mut status) },
                11.0
            );
            // Iterator time starts at "next time - 1", so we need to advance
            let _ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
            println!("ptime starts at {}", unsafe { *_ptime });
            assert_eq!(status, 0);

            // We iterate over PARENTAL generation times,
            // and only have work to do if any OFFSPRING demes
            // exist
            while unsafe { forward_graph_any_extant_offspring_demes(graph.as_ptr(), &mut status) } {
                assert_eq!(status, 0);

                assert!(!unsafe {
                    forward_graph_parental_deme_sizes(graph.as_ptr(), &mut status).is_null()
                });
                assert!(!unsafe {
                    forward_graph_offspring_deme_sizes(graph.as_ptr(), &mut status).is_null()
                });

                // Advance time to next PARENTAL generation
                let _ptime = unsafe { forward_graph_iterate_time(graph.as_mut_ptr(), &mut status) };
                // Update model internal state accordingly
                assert_eq!(
                    unsafe { forward_graph_update_state(*_ptime, graph.as_mut_ptr()) },
                    0
                );
                ngens += 1;
            }
            assert_eq!(ngens, (10.0 - start_time) as i32);
        }
    }

    #[test]
    fn test_model_with_bad_time_rounding() {
        let yaml = "
time_units: generations
demes:
- name: bad
  epochs:
  - {end_time: 1.5, start_size: 1}
  - {end_time: 0.4, start_size: 2}
  - {end_time: 0, start_size: 3}
";
        let mut graph = GraphHolder::new();
        assert_eq!(graph.init_with_yaml(10.0, yaml), 0);
        let x = graph.as_ptr();
        assert!(unsafe { forward_graph_is_error_state(x) });
    }

    #[test]
    fn test_non_integer_sizes_with_and_without_rounding() {
        let yaml = "
time_units: generations
demes:
- name: deme1
  start_time: .inf
  epochs:
  - {end_size: 99.99000049998334, end_time: 8000.0, start_size: 99.99000049998334}
  - {end_size: 100.0, end_time: 4000.0, start_size: 99.99000049998334}
  - {end_size: 100, end_time: 0, start_size: 100.0}
migrations: []
";
        {
            let mut graph = GraphHolder::new();
            assert_eq!(graph.init_with_yaml(10.0, yaml), 0);
            let x = graph.as_ptr();
            assert!(unsafe { forward_graph_is_error_state(x) });
        }
        {
            let mut graph = GraphHolder::new();
            assert_eq!(graph.init_with_yaml_round_epoch_sizes(10.0, yaml), 0);
            let x = graph.as_ptr();
            assert!(!unsafe { forward_graph_is_error_state(x) });
        }
    }
}
