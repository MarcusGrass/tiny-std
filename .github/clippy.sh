#!/bin/sh
cargo clippy -p rusl --no-default-features -- -D warnings
cargo clippy -p rusl --no-default-features --tests -- -D warnings
cargo clippy -p rusl -- -D warnings
cargo clippy -p rusl --tests -- -D warnings

cargo clippy -p tiny-std --no-default-features -- -D warnings
cargo clippy -p tiny-std --no-default-features --tests -- -D warnings
cargo clippy -p tiny-std -- -D warnings
cargo clippy -p tiny-std --tests -- -D warnings
cargo clippy -p tiny-std --features executable,threaded,global-allocator -- -D warnings