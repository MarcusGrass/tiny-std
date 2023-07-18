#!/bin/bash
set -ex
# Deny all warnings here, becomes a pain to scroll back otherwise
cargo clippy -p rusl --no-default-features -- -D warnings
cargo clippy -p rusl --no-default-features --tests -- -D warnings
cargo clippy -p rusl -- -D warnings
cargo clippy -p rusl --tests -- -D warnings
# test aarch64
cross clippy -p rusl --no-default-features --target aarch64-unknown-linux-gnu -- -D warnings
cross clippy -p rusl --no-default-features --target aarch64-unknown-linux-gnu --tests -- -D warnings
cross clippy -p rusl --target aarch64-unknown-linux-gnu -- -D warnings
cross clippy -p rusl --target aarch64-unknown-linux-gnu --tests -- -D warnings
