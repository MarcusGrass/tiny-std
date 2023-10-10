# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Fixed

### Added

### Changed
- Implement `From<TimeSpec>` for `SystemTime`

## [v0.2.2] - 2023-10-01

### Fixed
- Update tiny-start to avoid duplicated dependencies

## [v0.2.1] - 2023-10-01

### Changed
- Expose errors at crate root

## [v0.2.0] - 2023-10-01
### Fixed
- Create dir all used to error when it ended with a slash
- `args_os` started at second arg

### Added
- More tests

### Changed
- API-breakage from removing AsRefUnixStr and replacing with 
`&UnixStr`. Propagates down to all things containing `&UnixStr`. Main reason 
is making allocation more explicit on the user's side, inviting opportunities for 
const-evaluation of null-terminated strings.
- Un-publicked ReadBuf, might replace entirely later

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
