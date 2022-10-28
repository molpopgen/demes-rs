# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0-alpha.0] - 2022-10-28

### Bug Fixes

- [**breaking**] Graph::generation_time() now returns GenerationTime. ([#204](https://github.com/molpopgen/demes-rs/pull/204))

### Documentation

- Hide setup code in demes::load example ([#205](https://github.com/molpopgen/demes-rs/pull/205))
- Document json cargo feature ([#209](https://github.com/molpopgen/demes-rs/pull/209))

### Features

- Add several pub fns and an example. ([#201](https://github.com/molpopgen/demes-rs/pull/201))
- Add JSON support via cargo feature ([#206](https://github.com/molpopgen/demes-rs/pull/206))

### Miscellaneous Tasks

- Update github actions ([#208](https://github.com/molpopgen/demes-rs/pull/208))

### Styling

- Fix lints from nightly clippy ([#200](https://github.com/molpopgen/demes-rs/pull/200))

### Testing

- Use cargo hack to run tests ([#207](https://github.com/molpopgen/demes-rs/pull/207))

## [0.3.1] - 2022-10-27

### Miscellaneous Tasks

- Fix tag_pattern in cliff.toml ([#194](https://github.com/molpopgen/demes-rs/pull/194))
- Add dependabot update for github-actions. ([#195](https://github.com/molpopgen/demes-rs/pull/195))
- Bump actions/checkout from 2 to 3 ([#196](https://github.com/molpopgen/demes-rs/pull/196))
- Bump styfle/cancel-workflow-action from 0.6.0 to 0.11.0 ([#197](https://github.com/molpopgen/demes-rs/pull/197))

### Refactor

- Remove type name from impl Display for newtypes. ([#199](https://github.com/molpopgen/demes-rs/pull/199))

## [0.3.0] - 2022-10-25

### Documentation

- Remove link to private type ([#189](https://github.com/molpopgen/demes-rs/pull/189))

### Features

- Impl Display for Graph, add Graph::as_string ([#139](https://github.com/molpopgen/demes-rs/pull/139))
- Add Deme::most_recent_deme_start_time ([#191](https://github.com/molpopgen/demes-rs/pull/191))
- Add Epoch::start_time() ([#192](https://github.com/molpopgen/demes-rs/pull/192))
- [**breaking**] Add iterators over epoch times/sizes in a Deme ([#193](https://github.com/molpopgen/demes-rs/pull/193))

### Miscellaneous Tasks

- Git-cliff support ([#138](https://github.com/molpopgen/demes-rs/pull/138))
- Bump crate version to 0.3.0
- Update serde_yaml requirement from ~0.8 to ~0.9 ([#140](https://github.com/molpopgen/demes-rs/pull/140))
- Fix clippy lints ([#148](https://github.com/molpopgen/demes-rs/pull/148))
- Remove obsolete doc sections about panics ([#179](https://github.com/molpopgen/demes-rs/pull/179))

### Refactor

- Metadata now uses custom deserializer ([#142](https://github.com/molpopgen/demes-rs/pull/142))
- Remove use of Option<_> in AsymmetricMigration ([#145](https://github.com/molpopgen/demes-rs/pull/145))
- Remove possible panics in TryFrom<UnresolvedMigration> for Migration. ([#147](https://github.com/molpopgen/demes-rs/pull/147))
- Remove potential panics during Deme time resolution. ([#149](https://github.com/molpopgen/demes-rs/pull/149))
- Avoid copies when applying deme defaults ([#151](https://github.com/molpopgen/demes-rs/pull/151))
- Convert panics to Err in UnresolvedMigration. ([#153](https://github.com/molpopgen/demes-rs/pull/153))
- Conver panics to Err in impl Pulse. ([#154](https://github.com/molpopgen/demes-rs/pull/154))
- Convert panics to Err in impl Epoch ([#155](https://github.com/molpopgen/demes-rs/pull/155))
- Replace panic with Err in impl Deme ([#156](https://github.com/molpopgen/demes-rs/pull/156))
- Replace panic with Err in impl Graph ([#158](https://github.com/molpopgen/demes-rs/pull/158))
- [**breaking**] Metadata::as_yaml_string returns Result ([#160](https://github.com/molpopgen/demes-rs/pull/160))
- Remove code duplication from Pulse validation. ([#162](https://github.com/molpopgen/demes-rs/pull/162))
- Replace assert with Err during migration resolution.
- Streamline Epoch resolution ([#163](https://github.com/molpopgen/demes-rs/pull/163))
- Tidy up impl Deme ([#164](https://github.com/molpopgen/demes-rs/pull/164))
- Streamline Epoch size_function validation. ([#165](https://github.com/molpopgen/demes-rs/pull/165))
- Pass deme name and epoch index to epoch validation. ([#166](https://github.com/molpopgen/demes-rs/pull/166))
- [**breaking**] Separate Pulse and unresolved/HDM pulse types. ([#168](https://github.com/molpopgen/demes-rs/pull/168))
- Move migration resolution/validation fns to HDM types ([#170](https://github.com/molpopgen/demes-rs/pull/170))
- [**breaking**] Change internal storage for Deme. ([#172](https://github.com/molpopgen/demes-rs/pull/172))
- Remove use of itertools crate ([#174](https://github.com/molpopgen/demes-rs/pull/174))
- [**breaking**] Separate HDM/MDM demes ([#175](https://github.com/molpopgen/demes-rs/pull/175))
- Separate HDM/MDM Epoch types ([#177](https://github.com/molpopgen/demes-rs/pull/177))
- Remove last unwraps from Deme ([#178](https://github.com/molpopgen/demes-rs/pull/178))
- Settle on UnresolvedX for HDM types ([#180](https://github.com/molpopgen/demes-rs/pull/180))
- Streamline impl of UnresolvedEpoch ([#181](https://github.com/molpopgen/demes-rs/pull/181))
- Move time-related types to separate module ([#182](https://github.com/molpopgen/demes-rs/pull/182))
- Move MigrationRate to separate module ([#183](https://github.com/molpopgen/demes-rs/pull/183))
- Move SelfingRate to separate module ([#184](https://github.com/molpopgen/demes-rs/pull/184))
- Move CloningRate to separate module ([#185](https://github.com/molpopgen/demes-rs/pull/185))
- Move DemeSize to separte module ([#186](https://github.com/molpopgen/demes-rs/pull/186))
- Move Proportion to separate module ([#187](https://github.com/molpopgen/demes-rs/pull/187))
- Move TimeInterval to time.rs ([#188](https://github.com/molpopgen/demes-rs/pull/188))
- [**breaking**] Audit serde derive use ([#190](https://github.com/molpopgen/demes-rs/pull/190))

<!-- generated by git-cliff -->

## 2022-06-19, Version 0.2.2

### Commits
- [[`26d3426856`](https://github.com/molpopgen/demes-rs/commit/26d3426856cfb117c13fca8a34cbb17297fac9bb)] Bump version to 0.2.2 (Kevin R. Thornton)
- [[`9b103bcaf7`](https://github.com/molpopgen/demes-rs/commit/9b103bcaf740ed7ddf0adf7b9d81f85b20dc5c57)] Fix bug in converting to integer generations for (#129) (Kevin R. Thornton)
- [[`cac0a1c6ee`](https://github.com/molpopgen/demes-rs/commit/cac0a1c6ee2f863ddeb6de8cd8df0873d59f03c9)] Clean up comments from tests in specification.rs (#130) (Kevin R. Thornton)

## 2022-06-17, Version 0.2.1

Fix GitHub issues 126 and 127.

### Commits
- [[`a9b445f0a5`](https://github.com/molpopgen/demes-rs/commit/a9b445f0a527906b682b632422fd1f42a1797924)] Bump version to 0.2.1 (Kevin R. Thornton)
- [[`e2133cbfb7`](https://github.com/molpopgen/demes-rs/commit/e2133cbfb747373881cc50ba2ce6fd4189e22473)] Fix bugs in converting Graph to generations: (#128) (Kevin R. Thornton)

## 2022-06-17, Version 0.2.0
### Commits
- [[`fec9c650f2`](https://github.com/molpopgen/demes-rs/commit/fec9c650f2410f66cb2ba71d275a18bd798a3212)] rewrite main crate docs (#124) (Kevin R. Thornton)
- [[`8eab302e80`](https://github.com/molpopgen/demes-rs/commit/8eab302e803f9ca04b96dd95b04b123e4b7a3213)] Add Graph::to_integer_generations (#123) (Kevin R. Thornton)
- [[`f613234e8c`](https://github.com/molpopgen/demes-rs/commit/f613234e8c414d964c3d9f9a92dab3efa9af16ad)] Add Graph::to_generations() (#120) (Kevin R. Thornton)
- [[`a1860410e4`](https://github.com/molpopgen/demes-rs/commit/a1860410e41c1461d7d11ed944a03084661eb6eb)] Document specification.rs (#119) (Kevin R. Thornton)
- [[`4e4c762ea2`](https://github.com/molpopgen/demes-rs/commit/4e4c762ea22586d78b862ac11156759b2f46b70a)] document lib.rs (#118) (Kevin R. Thornton)
- [[`381bdf14a9`](https://github.com/molpopgen/demes-rs/commit/381bdf14a91a97c6960f98504df8db5145e993f5)] Document builder.rs (#113) (Kevin R. Thornton)
- [[`de22f5d9d7`](https://github.com/molpopgen/demes-rs/commit/de22f5d9d7018b24b9bad0f62fe304dda0e93eb3)] clean up commented-out code (#117) (Kevin R. Thornton)
- [[`abaa835e91`](https://github.com/molpopgen/demes-rs/commit/abaa835e91c6fc7fbc40f59b683eb448fdb68584)] Remove unnecessary uses of TryFrom (#116) (Kevin R. Thornton)
- [[`ca58cc4a61`](https://github.com/molpopgen/demes-rs/commit/ca58cc4a61929c886b117774f38cd61029874055)] All newtypes are now From<f64> instead of TryFrom. (#115) (Kevin R. Thornton)
- [[`5d28bb1959`](https://github.com/molpopgen/demes-rs/commit/5d28bb1959edb1f911d9310e1d64092783d26ca7)] Document DemesError (#111) (Kevin R. Thornton)
- [[`05240cddc1`](https://github.com/molpopgen/demes-rs/commit/05240cddc1b03b55fbbd81655faf49f14accfcf1)] Apply naming consitency to unresolved spec types. (#110) (Kevin R. Thornton)
- [[`ad6b86b6fd`](https://github.com/molpopgen/demes-rs/commit/ad6b86b6fdd681cc076f65a6cb40b59fcb9ad96d)] Separate resolved/unresolved Pulse using newtypes. (#109) (Kevin R. Thornton)
- [[`6a25123ff0`](https://github.com/molpopgen/demes-rs/commit/6a25123ff014940d10168bf2005976178827c597)] Define the public exports (#106) (Kevin R. Thornton)
- [[`fea785c90a`](https://github.com/molpopgen/demes-rs/commit/fea785c90a9f23a0903d6aa8aaa96695d4b894eb)] newtypes may now be compared to f64 (#105) (Kevin R. Thornton)
- [[`b6cf1157db`](https://github.com/molpopgen/demes-rs/commit/b6cf1157db2a9c6bad30e8377eaf065e88572466)] DemeData visibility changed to pub(crate). (#104) (Kevin R. Thornton)
- [[`b70787efae`](https://github.com/molpopgen/demes-rs/commit/b70787efae6d70b982ebf15fd880954781146a91)] GraphBuilder::add_deme no longer returns Result. (#103) (Kevin R. Thornton)
- [[`1003786d98`](https://github.com/molpopgen/demes-rs/commit/1003786d987b5906151968d07561554cdb32e065)] Add convenience constructor to GraphBuilder for time_units: generations (#102) (Kevin R. Thornton)
- [[`cdcc26ce07`](https://github.com/molpopgen/demes-rs/commit/cdcc26ce07b2e2104f8cb6be974c16270ccbb170)] GraphBuilder::new now accepts generation time and top-level defaults. (#101) (Kevin R. Thornton)
- [[`300be5a017`](https://github.com/molpopgen/demes-rs/commit/300be5a01757f80130cc39346f81a337670c7dd3)] Add GraphBuilder::add_pulse (#100) (Kevin R. Thornton)
- [[`6e6b9892fc`](https://github.com/molpopgen/demes-rs/commit/6e6b9892fcbfa264c287b3a7a066c432223b8c7f)] Add GraphBuilder::add_migration (#99) (Kevin R. Thornton)
- [[`8c03dcd8df`](https://github.com/molpopgen/demes-rs/commit/8c03dcd8df78f248a1c66a11fcf2990e5e146ca4)] Refactor Deme to reduce code duplication. (#98) (Kevin R. Thornton)
- [[`69071efcbe`](https://github.com/molpopgen/demes-rs/commit/69071efcbe536d96e8a711d553c1edd535e8caf7)] Add functionality to add a Deme to a Builder. (#89) (Kevin R. Thornton)
- [[`2d4c47b265`](https://github.com/molpopgen/demes-rs/commit/2d4c47b26542eed099de1acaa325e394771a2b7e)] Streamline internal code for serializing deme-level defaults. (#96) (molpopgen)
- [[`1b0f5bebaf`](https://github.com/molpopgen/demes-rs/commit/1b0f5bebafcde11474aa7c0dbd246d2a3bafbabb)] Add fn to return crate version. (#92) (Kevin R. Thornton)
- [[`9f137c2c03`](https://github.com/molpopgen/demes-rs/commit/9f137c2c03fdde426958c831c7a245002c6d2939)] Package version bump to 0.2.0 (molpopgen)
- [[`fb8b3d2e90`](https://github.com/molpopgen/demes-rs/commit/fb8b3d2e904bd97a9e0e4e92fdc2b5968f1f2810)] Use SnakeCase formatting for enum fields (#91) (Kevin R. Thornton)
- [[`19495067ac`](https://github.com/molpopgen/demes-rs/commit/19495067ac608a71db12600b9bd079a4904790f8)] Add minimal features to construct GraphBuilder. (#88) (Kevin R. Thornton)
- [[`2db891fc2b`](https://github.com/molpopgen/demes-rs/commit/2db891fc2b31ce0b7552e3348eb7e50af9203e42)] Merge GenerationTimeError into GraphError (#87) (Kevin R. Thornton)
- [[`8c0539e437`](https://github.com/molpopgen/demes-rs/commit/8c0539e437ea6b4c06b151a0c1309fd51d838e98)] rename ToplevelError to GraphError (#86) (Kevin R. Thornton)
- [[`1405ce86fc`](https://github.com/molpopgen/demes-rs/commit/1405ce86fc11b03543ad80c772a60233093c9138)] fix typo in README. (Kevin R. Thornton)
