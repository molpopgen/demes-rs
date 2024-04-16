# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2024-04-16

### Features

- Add ForwardGraph::demes_graph ([#376](https://github.com/molpopgen/demes-rs/pull/376))

### Miscellaneous Tasks

- Bump version number of all crates ([#354](https://github.com/molpopgen/demes-rs/pull/354))

### Styling

- Fix unnecessary use of fallible conversion ([#337](https://github.com/molpopgen/demes-rs/pull/337))
- Fix clippy lints for rust 1.75 ([#346](https://github.com/molpopgen/demes-rs/pull/346))

## [0.4.0] - 2023-09-26

### Documentation

- Fix broken intra-doc links ([#313](https://github.com/molpopgen/demes-rs/pull/313))

### Features

- Add demes-forward::ForwardGraph::size_at ([#264](https://github.com/molpopgen/demes-rs/pull/264))
- Impl Clone for demes::Graph and demes_forward::Graph ([#281](https://github.com/molpopgen/demes-rs/pull/281))
- Add ForwardGraph::deme_size_history ([#270](https://github.com/molpopgen/demes-rs/pull/270))
- Add utility fns for ForwardGraph ([#286](https://github.com/molpopgen/demes-rs/pull/286))
- Add fn to return deme names from graph ([#294](https://github.com/molpopgen/demes-rs/pull/294))

### Miscellaneous Tasks

- Bump MSRV to 1.60.0 ([#279](https://github.com/molpopgen/demes-rs/pull/279))

### Refactor

- [**breaking**] Mark SizeFunction non_exhaustive ([#258](https://github.com/molpopgen/demes-rs/pull/258))
- Graph::get_deme takes Into<DemeId> as argument ([#266](https://github.com/molpopgen/demes-rs/pull/266))
- [**breaking**] Improve strictness of all newtypes ([#272](https://github.com/molpopgen/demes-rs/pull/272))
- Use Vec instead of ndarray internally ([#275](https://github.com/molpopgen/demes-rs/pull/275))
- Fix invalid demes::Time during deme updates ([#276](https://github.com/molpopgen/demes-rs/pull/276))
- ForwardGraph::deme_size_history now works by explicit ([#288](https://github.com/molpopgen/demes-rs/pull/288))
- Use enums to abstract over input time types ([#293](https://github.com/molpopgen/demes-rs/pull/293))
- Demes_forward::Error uses thiserror decorators instead of custom impl ([#309](https://github.com/molpopgen/demes-rs/pull/309))

### Testing

- Test all size changes for test added in #253 ([#255](https://github.com/molpopgen/demes-rs/pull/255))

## [0.3.0] - 2023-03-30

### Refactor

- [**breaking**] Mark SizeFunction non_exhaustive ([#258](https://github.com/molpopgen/demes-rs/pull/258))

### Testing

- Test all size changes for test added in #253 ([#255](https://github.com/molpopgen/demes-rs/pull/255))

## [0.3.0-alpha.1] - 2023-03-14

### Bug Fixes

- Fix conversion to backwards time when all end_times > 0.0 ([#253](https://github.com/molpopgen/demes-rs/pull/253))

## [0.3.0-alpha.0] - 2023-03-13

### Bug Fixes

- Fix onset of, and formula for, exponential size change. ([#235](https://github.com/molpopgen/demes-rs/pull/235))
- Fix pulse/migration event timings. ([#238](https://github.com/molpopgen/demes-rs/pull/238))
- ForwardGraph::new Err on non-integer sizes ([#243](https://github.com/molpopgen/demes-rs/pull/243))

### Documentation

- Fix typo in crate-level documentation ([#224](https://github.com/molpopgen/demes-rs/pull/224))
- Update README.md for workspace and crates ([#226](https://github.com/molpopgen/demes-rs/pull/226))

### Miscellaneous Tasks

- Reorganize as a cargo workspace ([#215](https://github.com/molpopgen/demes-rs/pull/215))
- Update Cargo.toml to point to correct homepage/repo. ([#223](https://github.com/molpopgen/demes-rs/pull/223))

### Refactor

- [**breaking**] Mark DemesError and DemesForardError non_exhaustive ([#249](https://github.com/molpopgen/demes-rs/pull/249))
- [**breaking**] Handle time conversion/rounding with callbacks ([#251](https://github.com/molpopgen/demes-rs/pull/251))

### Testing

- Add test of ancestry proportions for demes with size 0. ([#230](https://github.com/molpopgen/demes-rs/pull/230))

## [0.2.1] - 2023-02-08

### Miscellaneous Tasks

- Bump Swatinem/rust-cache from 1 to 2 ([#75](https://github.com/molpopgen/demes-forward-rs/pull/75))
- Define crate MSRV ([#77](https://github.com/molpopgen/demes-forward-rs/pull/77))

### Styling

- Fix clippy lints ([#78](https://github.com/molpopgen/demes-forward-rs/pull/78))

## [0.2.0] - 2022-10-27

### Documentation

- First pass at docs for public API. ([#72](https://github.com/molpopgen/demes-forward-rs/pull/72))

### Features

- Add example program using Gutenkunst OOA model ([#73](https://github.com/molpopgen/demes-forward-rs/pull/73))

### Miscellaneous Tasks

- Bump version to 0.1.1 ([#61](https://github.com/molpopgen/demes-forward-rs/pull/61))
- Bump demes-rs to ~0.3.0 ([#60](https://github.com/molpopgen/demes-forward-rs/pull/60))
- Add cliff.toml ([#63](https://github.com/molpopgen/demes-forward-rs/pull/63))
- Fix link to crates.io ([#64](https://github.com/molpopgen/demes-forward-rs/pull/64))
- Bump version to 0.2.0 ([#66](https://github.com/molpopgen/demes-forward-rs/pull/66))
- Add dependabot.yml ([#67](https://github.com/molpopgen/demes-forward-rs/pull/67))
- Bump actions/checkout from 2 to 3 ([#68](https://github.com/molpopgen/demes-forward-rs/pull/68))
- Bump styfle/cancel-workflow-action from 0.6.0 to 0.11.0 ([#69](https://github.com/molpopgen/demes-forward-rs/pull/69))

### Refactor

- Use demes::Epoch::start_time() internally ([#65](https://github.com/molpopgen/demes-forward-rs/pull/65))
- Make ForwardGraph::model_times private. ([#70](https://github.com/molpopgen/demes-forward-rs/pull/70))
- [**breaking**] Remove super trait in favor of vanilla trait bounds ([#71](https://github.com/molpopgen/demes-forward-rs/pull/71))
- Replace unwraps with Err propagation. ([#74](https://github.com/molpopgen/demes-forward-rs/pull/74))

### Styling

- Fix clippy lints ([#62](https://github.com/molpopgen/demes-forward-rs/pull/62))

### Testing

- Add tests of iteration logic. ([#59](https://github.com/molpopgen/demes-forward-rs/pull/59))

<!-- generated by git-cliff -->
