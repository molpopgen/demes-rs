cmake_minimum_required(VERSION 3.15)
project(c_example LANGUAGES C)

MESSAGE(${CMAKE_SOURCE_DIR} ${PROJECT_SOURCE_DIR})

add_subdirectory(corrosion)
corrosion_import_crate(MANIFEST_PATH ../Cargo.toml)
get_filename_component(DEMES_FORWARD_HEADER_LOCATION ${CMAKE_CURRENT_SOURCE_DIR} DIRECTORY CACHE)
add_custom_target(header DEPENDS ${DEMES_FORWARD_HEADER_LOCATION}/demes_forward.h)
add_executable(example example.c)
add_dependencies(example cargo-build_demes_forward_capi header)
target_include_directories(example BEFORE PUBLIC ${DEMES_FORWARD_HEADER_LOCATION})
target_link_directories(example PUBLIC ${CMAKE_BINARY_DIR})
target_link_libraries(example PUBLIC demes_forward_capi)

add_custom_command(OUTPUT ${DEMES_FORWARD_HEADER_LOCATION}/demes_forward.h COMMAND cbindgen -l C --cpp-compat -o ${DEMES_FORWARD_HEADER_LOCATION}/demes_forward.h ${CMAKE_SOURCE_DIR}/..)
