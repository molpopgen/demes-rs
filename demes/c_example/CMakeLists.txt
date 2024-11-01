# Usage from the workspace root:
# cmake -S demes/c_example -B build
# cmake --build build
# cmake --build build --target clean
cmake_minimum_required(VERSION 3.15)
project(c_example LANGUAGES C)

MESSAGE(${CMAKE_SOURCE_DIR} ${PROJECT_SOURCE_DIR})

add_compile_options(-W -Wall -Werror -Wconversion)
add_subdirectory(corrosion)

# Specify to only build the demes crate and to use the ffi cargo feature of the crate
corrosion_import_crate(MANIFEST_PATH ../Cargo.toml CRATES demes FEATURES ffi)
# The header, demes.h, will be built in the root of the build dir
get_filename_component(DEMES_HEADER_LOCATION ${CMAKE_BINARY_DIR} DIRECTORY CACHE)
add_custom_target(header DEPENDS ${DEMES_HEADER_LOCATION}/demes.h)
add_executable(example example.c)
add_executable(example_using_iterators example_using_iterators.c)
add_dependencies(example cargo-build_demes header)
target_include_directories(example BEFORE PUBLIC ${DEMES_HEADER_LOCATION})
target_link_directories(example PUBLIC ${CMAKE_BINARY_DIR})
# We link the static C archive of demes and the C math lib to the binary
target_link_libraries(example PUBLIC libdemes.a m)
add_dependencies(example_using_iterators cargo-build_demes header)
target_include_directories(example_using_iterators BEFORE PUBLIC ${DEMES_HEADER_LOCATION})
target_link_directories(example_using_iterators PUBLIC ${CMAKE_BINARY_DIR})
# We link the static C archive of demes and the C math lib to the binary
target_link_libraries(example_using_iterators PUBLIC libdemes.a m)

# Use cbindgen to build our header
add_custom_command(OUTPUT ${DEMES_HEADER_LOCATION}/demes.h COMMAND cbindgen -l C -o ${DEMES_HEADER_LOCATION}/demes.h ${CMAKE_SOURCE_DIR}/..)
