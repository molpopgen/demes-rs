# demes-rs

![CI tests](https://github.com/molpopgen/demes-rs/workflows/CI/badge.svg)

[rust](https://www.rustlang.org) tools for the 
[demes](https://popsim-consortium.github.io/demes-spec-docs/main/introduction.html#sec-intro)
specification.

This repository contains the following rust crates:

* [demes](https://crates.io/crates/demes) implements the specification and a graph builder.
* [demes-forward](https://crates.io/crates/demes-forward) provides a means to handle graphs forwards in time.
* [demes-forward-capi](https://crates.io/crates/demes-forward-capi) is a C interface to `demes-foward`.

## Developer information

### Cloning the repository and running the test suite

```sh
git clone https://github.com/molpopgen/demes-rs
cd demes-rs
cargo test
```

### Pull requests

* Pull requests should be rebased down to one commit.
* Commit messages for CHANGELOGs should be
  [conventional](https://www.conventionalcommits.org/en/v1.0.0/).
* We **strongly** suggest running semver checks locally.
  While we run these upon merge into `main`, it is better to know
  ahead of time if a PR breaks API.   It is even better to
  avoid API breakage altogether!

```sh
cargo install cargo-semver-checks
cargo semver-checks check-release
```

### Generating CHANGELOG updates

* We use [git-cliff](https://github.com/orhun/git-cliff)
* To update a CHANGELOG for a given crate, use include paths.
  For example:

```sh
git cliff -u --include-path "demes/**" --tag v0.4.0 -p demes/CHANGELOG.md
```

The configuration file for cliff is present in the workspace root.

### Tagging a release

Any new release of any tool should have the following name:

```
tool-version
```

For example:

```
demes-forward-0.6.0
```

### Viewing the documentation locally

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

