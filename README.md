# demes-rs

![CI tests](https://github.com/molpopgen/demes-rs/workflows/CI/badge.svg)

A [rust](https://www.rustlang.org) implementation of the [demes](https://popsim-consortium.github.io/demes-spec-docs/main/introduction.html#sec-intro) specification.

## Developer information

### Cloning the repository and running the test suite

```sh
git clone https://github.com/molopgen/demes-rs
cd demes-rs
cargo test
```

### Viewing the documentation

```
cargo doc --open
```

### Calculating code coverage

First, install `tarpaulin`:

```sh
cargo install cargo-tarpaulin
```

Then,

```sh
cargo tarpaulin --tests --ignore-tests -o html
```

Finally, open `tarpaulin-report.html` with your favorite browser.
