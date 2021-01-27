# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

- Updated the `dusk-plonk` versions to `v0.2.7`.

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

- Refactored the tests to work with the new ZK-Proof algorithm Plonk.

### Fixed

- Reduced the size of the circuit produced by reducing some gates that could be merged.

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

### Added

- Basic Hades252 implementation.
- Strategy structure.
- Use `curve25519-dalek` as curve-backend.
- Use `bulletproofs` as proving system.
