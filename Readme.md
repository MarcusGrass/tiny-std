# Tiny std

Like a bad, probably buggy, tiny standard library for Linux.

## NOTICE: Broken v0.1.0 on stable >= 1.72 / nightly >= 08-17
Some update to rustc between 2023-08-16 and 2023-08-17 caused miscompilations of 
tiny-std resulting in infinite recursion on calls to `memset`, which the compiler 
inserts sometimes as part of optimization [a bit more details here](https://github.com/rust-lang/rust/issues/115225#issuecomment-1705196246).  
This was fixed in v0.1.1 with no API-changes by breaking code sensitive to rewrites into 
a separate crate with a `#![no_builtins]`.  

Therefore, version `0.1.0` was yanked from crates.io.

## When it's appropriate
If you are actually trying to do something solid,
checkout [Rustix](https://github.com/bytecodealliance/rustix) or [Relibc](https://github.com/redox-os/relibc).  

The regular stdlib is probably going to be better for almost all use-cases, since the only supported os is Linux 
you likely have an allocator present, although tiny-std will in my testing at least, produce a much smaller binary.  


# Supported platforms

1. x86_64
2. aarch64

# Core features

1. Run with or without alloc
2. Minimal fs coverage
3. Minimal spawn coverage
4. Minimal unix socket support
5. Minimal threading support
6. Experimental io_uring support

# Wanted features (in no particular order)

1. Signal handling by signalfd
2. Threading would be nice, but reinventing that particular wheel will likely explode code footprint 
and be hard to get right.
3. io-uring fs operations

# Examples
- [PGWM](https://github.com/MarcusGrass/pgwm) is the biggest project built with `tiny-std`.  
At present, the minimal WM builds statically pie-linked at `790K`.  
- Some examples of working setups for binary projects are in [test-runners](./test-runners), 
both with and without an allocator/threading.  


## License

This project and any contributions are licensed under [MPL-2.0](LICENSE).
A lot of code is directly lifted from rust-std, not mentioning the API which is meant to be similar/compatible
as much as possible with rust-std, that's licensed under MIT and can be found
here [rust-std-MIT](tiny-std/STDLIB_LICENSE).