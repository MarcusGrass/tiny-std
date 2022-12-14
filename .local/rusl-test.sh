#!/bin/sh
set -ex
cargo test -p rusl --no-default-features
cargo test -p rusl

cross test -p rusl --no-default-features --target aarch64-unknown-linux-gnu
cross test -p rusl --target aarch64-unknown-linux-gnu
