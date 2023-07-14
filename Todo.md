# Todo

1. [x] Replace usage of linux-syscalls and delete it
2. [x] Fixup start
3. [x] Fix signal handling
4. [ ] Should probably use signalfd instead for less wild unsafety
5. [ ] Generate and sort raw types directly from kernel code
6. [x] Figure out VDSO for `x86_64`
7. [x] Figure out VDSO for `aarch64`, currently not getting the aux value
8. [x] Get a real mutex (got rwlock)
9. [ ] Feature gate things even harder in both [rusl](rusl) and [tiny-std](tiny-std)
10. [ ] Generate debug info depending on opcode for io uring sqes
11. [ ] Implement file copy, should probably copy mode
