# demes-forward-capi

This crate provides a C interface to [demes-forward](https://docs.rs/demes-forward/).

In general, you should prefer the other crate unless you are working in C/C++.

This crate generates:

1. A header file
2. A static C library
3. A dynamic C library

## Requirements

[cbindgen](https://github.com/eqrion/cbindgen) generates the header file.
We do not use a [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
Rather, we use `cbindgen` from the command line.

To install:

```sh
cargo install cbindgen
```

## Integration with `cmake`.

See `c_examples/` in the [repository](https://github.com/molpopgen/demes-forward-capi).
