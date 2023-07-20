#!/bin/sh
cargo test -p rusl --no-default-features -- --test-threads=1
cargo test -p rusl -- --test-threads=1

cargo test -p tiny-std --no-default-features
cargo test -p tiny-std --features threaded,global-allocator
