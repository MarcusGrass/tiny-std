# Tiny std

Like a bad, probably buggy, tiny standard library for Linux.

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

## License

This project and any contributions are licensed under [MPL-2.0](LICENSE).
A lot of code is directly lifted from rust-std, not mentioning the API which is meant to be similar/compatible
as much as possible with rust-std, that's licensed under MIT and can be found
here [rust-std-MIT](tiny-std/STDLIB_LICENSE).