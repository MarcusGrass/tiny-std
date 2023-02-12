#!/bin/sh
set -ex
cargo test -p rusl --no-default-features -- --test-threads=1
cargo test -p rusl -- --test-threads=1

cross test -p rusl --no-default-features --target aarch64-unknown-linux-gnu -- --test-threads=1
cross test -p rusl --target aarch64-unknown-linux-gnu -- --test-threads=1
