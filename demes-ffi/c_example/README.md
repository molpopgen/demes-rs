# Example of using the C API

## Requirements

* Rust.  See [here](https://www.rust-lang.org/tools/install)
* cbindgen.  Once rust is installed:

    ```
    cargo install cbindgen --locked
    ```
* cmake
* [corrosion](https://github.com/corrosion-rs/corrosion)

## Building

From this directory (`demes/c_example`):

```
cmake -B../build -S.
cmake --build ../build
```

The previous command builds a debug build.
For an optimized build:

```
cmake -B../build -S. -DCMAKE_BUILD_TYPE=Release
cmake --build ../build
```

## Notes for developers using the C API

It is critical to read the contents of the generated header file!
The documentation for each function documents how to use each function correctly and how to avoid undefined behavior.

It is important to understand the ownership of memory!
(This is covered in the documentation for each function.)
Some function return pointers to newly-allocated memory.
The functions needed to "free" such memory are documented.
Further note that pointers to newly-allocated `char *` must be freed by a specific function.
(Again, see the generated header).
This function is NOT the C `free` function!

C/C++ programmers may be surprised to see that their final binary sizes are quite large.
This size is because compiled rust code results in self-contained objects that are generally not dynamically linked to anything else.


## Limitations

Iteration over demes, epochs, etc., is currently only supported via `for` loops.
We may add opaque structs that model iterator behavior.
