# Changelog

## 1.0.0 - 2018-10-06

### Changed

- Update license.
- Update documentation.
- Better error messages.
- Put trait bounds on member functions instead of struct.
- Use `impl Trait` feature.
- Use range expressions features.

## 0.3.0 - 2018-05-11

### Changed

- Use `impl Trait` feature.

## 0.2.3 - 2018-04-30

### Fixed

- Fix build failure on `stable` due to backwards incompatible feature.

## 0.2.2 - 2018-04-30

### Changed

- Update formatting.
- Replace `TreapMap` with `BTreeMap`.

## 0.2.1 - 2018-04-23

### Changed

- Update tests and formatting.
- Exclude benchmarks from code coverage.
- Remove unneeded `mut` in `mpc::Ring::get_node`.

## 0.2.0 - 2018-04-22

### Added

- Example client usages.
- `jump` module with `Ring`.
- `maglev` module with `Ring`.
- `mpc` module with `Ring`.
- Benchmarks and distribution tests.

### Changed

- Use `SipHasher` instead of xor for hashing.

## 0.1.0 - 2018-04-05

### Added

- `carp` module with `Node`, and `Ring`.
- `consistent` module with `Ring`, and `Client`.
- `rendezvous` module with `Ring`, and `Client`.
- `weighted_rendezvous` module with `Ring`, and `Client`.
