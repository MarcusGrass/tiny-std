#!/bin/sh
set -ex
cargo clippy -p tiny-std --no-default-features -- -D warnings
cargo clippy -p tiny-std -- -D warnings

cross clippy -p tiny-std --no-default-features --target aarch64-unknown-linux-gnu -- -D warnings
cross clippy -p tiny-std --target aarch64-unknown-linux-gnu -- -D warnings

cargo test -p tiny-std --no-default-features
cargo test -p tiny-std

cross test -p tiny-std --no-default-features --target aarch64-unknown-linux-gnu
cross test -p tiny-std --target aarch64-unknown-linux-gnu