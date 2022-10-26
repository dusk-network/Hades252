# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.20.0] - 2022-10-26

### Changed

- Update `dusk-plonk` from `0.12` to `0.13`

## [0.19.0] - 2022-08-17

### Changed

- Update `dusk-bls12_381` from `0.8` to `0.11`
- Update `dusk-plonk` from `0.9` to `0.12`

## [0.16.0] - 2021-07-02

### Added

- Add toolchain file set to nightly-2021-06-06 [#99]

### Changed

- Change CI toolchain to not override toolchain-file [#99]
- Update `dusk-bls12_381` & `dusk-plonk` to latest versions. [#84]
- Replace `rand` by `rand-core` [#86]
- Change repo to `no_std` with `alloc` feature [#87]
- Change `ROUND_CONSTANTS` and `MDS_MATRIX` to be generated at compile time [#96]

### Removed

- Remove `anyhow` from dev-deps [#85]
- Remove `lazy_static` as dependency

### Fixed

- Fix `Readme.md` import into lib docs [#98]

## [0.15.0] - 2021-04-06

### Changed

- Update `plonk` from `0.6` to `v0.7` #81

## [0.14.0] - 2021-03-11

### Changed

- Update `plonk` from `0.5` to `v0.6` #79

## [0.13.0] - 2021-02-11

### Changed

- Change crate's name from `Hades252` to `dusk-hades`

## [0.12.0] - 2021-01-27

### Changed

- Bump `dusk-plonk` to `v0.5`
- Bump `dusk-bls12_381` to `v0.6`
- Remove `nightly_docs` feature

## [0.11.0] - 2021-01-26

### Changed

- Bump `dusk-plonk` to `v0.4`
- Bump `dusk-bls12_381` to `v0.5`
- Update CHANGELOG to ISO 8601

## [0.10.1] - 2020-11-09

### Changed

- Update to support full no-std.

## [0.10.0] - 2020-10-06

### Fixed

- Fixes #69 - ARK on partial round must be applied to all elements.

## [0.9.0] - 2020-10-05

### Changed

- Major optimization on `GadgetStrategy` with fan-in-3 feature of PLONK.
  Gates per permutation set to `973`

## [0.8.1] - 2020-10-04

### Changed

- Optimize `GadgetStrategy` to consume less gates.

## [0.8.0] - 2020-09-29

### Changed

- Bump `dusk-plonk` to `v0.2.11`.

## [0.7.0] - 2020-08-13

### Added

- `anyhow` crate to support Error handling in the tests.

### Changed

- Update the `dusk-plonk` versions to `v0.2.7`.

### Removed

- Legacy methods to perform `poseidon-based ops` such as hashing
  which is not the purpose of this lib.

## [0.6.1] - 2020-07-24

### Changed

- `dusk-plonk` crate version to `0.2.1`.

## [0.6.0] - 2020-07-21

### Changed

- `dusk-plonk` crate version to `v0.2.0`
- Tests for gadgets now use the Prover&Verifier abstraction.

### Removed

- `dusk-bls12_381` deps which are now taken from plonk's re-exported ones.

## [0.5.0] - 2020-05-11

### Added

- `dusk-plonk_v0.1.0` as proving system.
- `dusk-bls12_381_v0.1.0` as curve-backend

### Changed

- `GadgetStrategy` structure refactor & optimization.
- tests updated & refactored with the new proving system.

### Removed

- `Bulletproofs` is no longer the proving system used.
- `Curve25519-dalek` is no longer used as curve-backend.

## [0.4.0] - 2020-04-12

### Added

- Plonk/fast_prover_zexe backend for ZK-Gadget functions
- Algebra, poly_commit & num_traits from Zexe as dependencies to work with PLONK.

### Changed

- Refactor the tests to work with the new ZK-Proof algorithm Plonk.

### Fixed

- Reduce the size of the circuit produced by reducing some gates that could be merged.

### Removed

- Bulletproofs dependencies removal.
- Curve25519 dependencies removal.

## [0.3.0] - 2020-03-21

### Changed

- Bulletproofs dependencies change to use dusk-network/bulletproofs "branch=develop".

## [0.2.0] - 2020-03-15

### Changed

- Bulletproofs dependencies change to use dusk-network/bulletproofs "branch=dalek_v2".

## [0.1.0] - 2020-02-27

### Addeded

- Basic Hades252 implementation.
- Strategy structure.
- Use `curve25519-dalek` as curve-backend.
- Use `bulletproofs` as proving system.

[#99]: https://github.com/dusk-network/Hades252/issues/99
[#98]: https://github.com/dusk-network/Hades252/issues/98
[#96]: https://github.com/dusk-network/Hades252/issues/96
[#87]: https://github.com/dusk-network/Hades252/issues/87
[#86]: https://github.com/dusk-network/Hades252/issues/86
[#85]: (https://github.com/dusk-network/Hades252/issues/85)
[#84]: https://github.com/dusk-network/Hades252/issues/84
[unreleased]: https://github.com/dusk-network/dusk-abi/compare/v0.20.0...HEAD
[0.20.0]: https://github.com/dusk-network/dusk-abi/compare/v0.19.0...v0.20.0
[0.19.0]: https://github.com/dusk-network/dusk-abi/compare/v0.16.0...v0.19.0
[0.16.0]: https://github.com/dusk-network/dusk-abi/compare/v0.15.0...v0.16.0
[0.15.0]: https://github.com/dusk-network/dusk-abi/compare/v0.14.0...v0.15.0
[0.14.0]: https://github.com/dusk-network/dusk-abi/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/dusk-network/dusk-abi/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/dusk-network/dusk-abi/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/dusk-network/dusk-abi/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/dusk-network/dusk-abi/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/dusk-network/dusk-abi/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/dusk-network/dusk-abi/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/dusk-network/dusk-abi/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/dusk-network/dusk-abi/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/dusk-network/dusk-abi/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/dusk-network/dusk-abi/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/dusk-network/dusk-abi/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/dusk-network/dusk-abi/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/dusk-network/dusk-abi/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/dusk-network/dusk-abi/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/dusk-network/dusk-abi/compare/v0.1.0...v0.2.0
[0.1.1]: https://github.com/dusk-network/dusk-abi/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dusk-network/dusk-abi/releases/tag/v0.1.0
