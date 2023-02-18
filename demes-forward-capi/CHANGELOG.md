# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2022-10-27

### Bug Fixes

- Fix tag_version in cliff.toml

### Miscellaneous Tasks

- Bump demes-forward version to ~0.2 ([#35](https://github.com/molpopgen/demes-forward-capi/pull/35))
- Add .github/dependabot.yml ([#37](https://github.com/molpopgen/demes-forward-capi/pull/37))
- Bump crate version to 0.3.0 ([#36](https://github.com/molpopgen/demes-forward-capi/pull/36))
- Bump jwlawson/actions-setup-cmake from 1.12 to 1.13 ([#38](https://github.com/molpopgen/demes-forward-capi/pull/38))
- Bump actions/checkout from 2 to 3 ([#39](https://github.com/molpopgen/demes-forward-capi/pull/39))
- Bump styfle/cancel-workflow-action from 0.6.0 to 0.11.0 ([#40](https://github.com/molpopgen/demes-forward-capi/pull/40))

### Styling

- Cargo +nightly clippy --fix ([#41](https://github.com/molpopgen/demes-forward-capi/pull/41))

### Testing

- Add tests to help establish iteration patterns. ([#33](https://github.com/molpopgen/demes-forward-capi/pull/33))
- Test that deme size pointers aren't NULL. ([#34](https://github.com/molpopgen/demes-forward-capi/pull/34))

## [0.2.0] - 2022-08-07

### Miscellaneous Tasks

- Bump crate version to 0.2.0
- Add cliff.toml

### Refactor

- Build the normal rust lib. Facilitates re-exports. ([#32](https://github.com/molpopgen/demes-forward-capi/pull/32))

<!-- generated by git-cliff -->