# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `dusk-plonk_v0.1.0` as proving system.
- `dusk-bls12_381_v0.1.0` as curve-backend

### Changed
- `GadgetStrategy` structure refactor & optimization.
- tests updated & refactored with the new proving system.

### Removed
- `Bulletproofs` is no longer the proving system used.
- `Curve25519-dalek` is no longer used as curve-backend.


## [0.1.0] - 27-02-20

### Added

- Basic Hades252 implementation.
- Strategy structure.
- Use `curve25519-dalek` as curve-backend.
- Use `bulletproofs` as proving system.

### Changed

### Fixed

### Removed
