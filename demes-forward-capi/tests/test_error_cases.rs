use demes_forward_capi::*;

#[test]
fn test_initialize_from_non_existant_file() {
    let graph = forward_graph_allocate();
    let filename = "no_way_this_exists";
    unsafe {
        forward_graph_initialize_from_yaml_file(filename.as_ptr() as *const i8, 100.0, graph)
    };

    let is_error = unsafe { forward_graph_is_error_state(graph) };
    assert!(is_error);

    unsafe {
        forward_graph_deallocate(graph);
    }
}

#[test]
fn test_errors_const_api_with_uninitialized_graph() {
    let graph = forward_graph_allocate();
    let mut status = 0;
    let _ = unsafe { forward_graph_selfing_rates(graph as *const OpaqueForwardGraph, &mut status) };
    assert!(status < 0); // make sure we are in an error state

    status = 0;
    let _ = unsafe { forward_graph_cloning_rates(graph as *const OpaqueForwardGraph, &mut status) };
    assert!(status < 0);

    status = 0;
    let _ = unsafe {
        forward_graph_parental_deme_sizes(graph as *const OpaqueForwardGraph, &mut status)
    };
    assert!(status < 0);

    status = 0;
    let _ = unsafe {
        forward_graph_offspring_deme_sizes(graph as *const OpaqueForwardGraph, &mut status)
    };
    assert!(status < 0);

    status = 0;
    let _ = unsafe {
        forward_graph_any_extant_parent_demes(graph as *const OpaqueForwardGraph, &mut status)
    };
    assert!(status < 0);

    status = 0;
    let _ = unsafe {
        forward_graph_any_extant_offspring_demes(graph as *const OpaqueForwardGraph, &mut status)
    };
    assert!(status < 0);

    status = 0;
    let _ =
        unsafe { forward_graph_model_end_time(graph as *const OpaqueForwardGraph, &mut status) };
    assert!(status < 0);

    unsafe {
        forward_graph_deallocate(graph);
    }
}

// Test that fns expecting *const OpaqueForwardGraph
// Return an error if the pointer is null
macro_rules! make_test_of_const_api_with_null {
    ($name: ident, $function: ident) => {
        #[test]
        fn $name() {
            let mut status = 0;
            unsafe {
                $function(std::ptr::null() as *const OpaqueForwardGraph, &mut status);
            }
            assert!(status < 0);
        }
    };
}

make_test_of_const_api_with_null!(
    test_ub_any_extant_offspring_demes,
    forward_graph_any_extant_offspring_demes
);
make_test_of_const_api_with_null!(
    test_ub_any_extant_parent_demes,
    forward_graph_any_extant_parent_demes
);
make_test_of_const_api_with_null!(
    test_ub_any_offspring_deme_sizes,
    forward_graph_offspring_deme_sizes
);
make_test_of_const_api_with_null!(
    test_ub_any_parental_deme_sizes,
    forward_graph_parental_deme_sizes
);
make_test_of_const_api_with_null!(test_ub_cloning_rates, forward_graph_cloning_rates);
make_test_of_const_api_with_null!(test_ub_selfing_rates, forward_graph_selfing_rates);

fn simple_yaml() -> std::ffi::CString {
    let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
    std::ffi::CString::new(yaml).unwrap()
}

#[test]
fn test_initialize_from_yaml_with_null_graph() {
    let cstr = simple_yaml();
    let status =
        unsafe { forward_graph_initialize_from_yaml(cstr.as_ptr(), 100.0, std::ptr::null_mut()) };
    assert!(status < 0);
}
