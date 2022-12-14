#!/bin/sh
set -ex

cargo test -p tiny-std --no-default-features
cargo test -p tiny-std

cross test -p tiny-std --no-default-features --target aarch64-unknown-linux-gnu
cross test -p tiny-std --target aarch64-unknown-linux-gnu