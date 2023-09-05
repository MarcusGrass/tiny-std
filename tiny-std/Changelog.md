# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).
## [Unreleased]
### Fixed

### Added

### Changed

## [v0.1.1]
### Fixed
- Segfaults on binaries built by rust 1.72+ and `nightly-08-16+` [details](https://github.com/rust-lang/rust/issues/115225#issuecomment-1694183173) by 
breaking out symbols into a separate `#![no_builtins]` lib. Also solves potential future breakage 
from llvm inserting `memset` on code running before symbol relocations.

### Added
- None

### Changed
- None

## [v0.1.0] - 2023-07-25

### Added
- Initial `tiny-std`-version
