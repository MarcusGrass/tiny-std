# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).
## [0.5.0] - 2026-02-12
### Fixed

### Added

### Changed

- Change io-uring SQE-slot handling to be a guard-struct which is more difficult to misuse

## [0.4.0] - 2025-02-08

### Fixed

### Added

- Get the ipv4 address of a `SocketAddressInet`

### Changed

- SocketAddressInet is now created from le_bytes, which is generally
how one would write a 4 byte ipv4 in an array i.e: `[127, 0, 0, 1]`


## [0.3.1] - 2024-08-24

### Fixed

### Added
- Implement GETPID syscall
- Implement READV syscall
- implement WRITEV syscall

### Changed

## [0.3.0] - 2024-05-05

### Fixed

### Added

- TCP-sockets implementation
- TCP-sockets implementation for io-uring
- Bunch of tests
- TCP and Unix socket `GETSOCKETNAME` implementation
- Sendmsg and Recvmsg implementations - Very rough implementation, subject to improvement.
- PollAdd io-uring implementation
- lseek implementation

### Changed

- Rename socket functions to be domain-specific

## [v0.2.2] - 2023-11-07
### Added
- Utility methods for `UnixStr` to make it easier to navigate them
as paths
- Find-method for `UnixStr`
- Accessors for some inner fields of `Statx`
- `unix_lit!` macros

## [v0.2.1] - 2023-10-01

### Changed
- Throw a rusl error instead of a Utf8Error on failed `UnixStr` conversions.

## [v0.2.0] - 2023-10-01

### Changed
- API-breakage from removing AsRefUnixStr and replacing with 
`&UnixStr`. Propagates down to all things containing `&UnixStr`. Main reason 
is making allocation more explicit on the user's side, inviting opportunities for 
const-evaluation of null-terminated strings.

## [v0.1.0] - 2023-07-25

### Added
- Initial `rusl`-version
