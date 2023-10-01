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
12. [ ] Figure out whether it should be possible to run start without `aux`, since that configuration
makes static-pie binaries not work properly.
13. [ ] Figure out whether there's a comparable `no_std`, `no-libc` allocator that's more suitable 
for applications that can be threaded.  
14. [ ] Use more efficient syscall semantics, i.e. `eax` over `rax` if the return-value isn't register size.  
15. [ ] Use type-checked builders as args for comptime error evaluation of syscalls.  
16. [ ] Enforce correct features for symbol relocation through a build-script (Fail compilation with 
static relocation if `aux` feature isn't enabled, since that will result in a botched binary).  
17. [ ] Path operations on &UnixStr
18. [x] Implement `from_str` on &UnixStr
19. [x] Throw a rusl::Error instead of Utf8Error on `as_str`