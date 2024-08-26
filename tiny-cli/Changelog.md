# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Fixed

### Added

### Changed

## [v0.3.1] - 2024-08-26

### Fixed
- Correctly handles different visibilities on Arg structs

## [v0.3.0] - 2024-08-24

### Fixed
- Some aliases not registering properly

### Added
- Positional arguments now accepted, and the default for untagged fields

### Changed
- Bools are always optional and should never be specified as `True` or `False`
- Options must now be specified as either `long` or `short` with their alias

## [v0.2.1] - 2024-05-05
### Fixed

- Update tiny-std

## [v0.2.0] - 2023-11-07

### Changed
- Tiny-cli ArgParser as proc macro instead of regular macro
