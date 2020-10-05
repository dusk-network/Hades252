# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.1] - 04-10-20
### Changed
- Optimize `GadgetStrategy` to consume less gates.

## [0.8.0] - 29-09-20
### Changed
- Bump `dusk-plonk` to `v0.2.11`.

## [0.7.0] - 13-08-20
### Added
- `anyhow` crate to support Error handling in the tests.

### Changed
- Updated the `dusk-plonk` versions to `v0.2.7`.

### Removed
- Legacy methods to perform `poseidon-based ops` such as hashing
which is not the purpose of this lib.


## [0.6.1] - 24-07-20
### Changed
- `dusk-plonk` crate version to `0.2.1`.

## [0.6.0] - 21-07-20
### Changed
- `dusk-plonk` crate version to `v0.2.0`
- Tests for gadgets now use the Prover&Verifier abstraction.

### Removed
- `dusk-bls12_381` deps which are now taken from plonk's re-exported ones.

## [0.5.0] - 11-05-20

### Added
- `dusk-plonk_v0.1.0` as proving system.
- `dusk-bls12_381_v0.1.0` as curve-backend

### Changed
- `GadgetStrategy` structure refactor & optimization.
- tests updated & refactored with the new proving system.

### Removed
- `Bulletproofs` is no longer the proving system used.
- `Curve25519-dalek` is no longer used as curve-backend.

## [0.4.0] - 12-04-20

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


## [0.3.0] - 21-03-20

### Changed
- Bulletproofs dependencies change to use dusk-network/bulletproofs "branch=develop".



## [0.2.0] - 15-03-20

### Changed
- Bulletproofs dependencies change to use dusk-network/bulletproofs "branch=dalek_v2".

## [0.1.0] - 27-02-20

### Added

- Basic Hades252 implementation.
- Strategy structure.
- Use `curve25519-dalek` as curve-backend.
- Use `bulletproofs` as proving system.
