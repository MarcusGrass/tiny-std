# Tiny std

Like a bad, probably buggy, tiny standard library for Linux.

If you are actually trying to do something solid,
checkout [Rustix](https://github.com/bytecodealliance/rustix) or [Relibc](https://github.com/redox-os/relibc).

# Supported platforms

1. x86_64
2. aarch64

# Core features

1. Run with or without alloc
2. Minimal fs coverage
3. Minimal spawn coverage
4. Minimal unix socket support

# Wanted features

1. An allocator, currently [dl-malloc-rs](https://github.com/alexcrichton/dlmalloc-rs) can be rewritten no-libc
   pretty [easily as is done here](https://github.com/marcusGrass/dlmalloc-rs)
2. An RwLock, lifting that from rust-std is possible but the code footprint is pretty large
3. Signal handling by signalfd

## License

The project is licensed under [MPL-2.0](LICENSE).
A lot of code is directly lifted from rust-std, not mentioning the API which is meant to be similar/compatible
as much as possible with rust-std, that's licensed under MIT and can be found
here [rust-std-MIT](tiny-std/STDLIB_LICENSE).