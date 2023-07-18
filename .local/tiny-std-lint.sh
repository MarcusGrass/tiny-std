#!/bin/sh
set -ex
cargo clippy -p tiny-std --no-default-features -- -D warnings
cargo clippy -p tiny-std --no-default-features --tests -- -D warnings
cargo clippy -p tiny-std -- -D warnings
cargo clippy -p tiny-std --tests -- -D warnings
cargo clippy -p tiny-std --features executable -- -D warnings

cross clippy -p tiny-std --no-default-features --target aarch64-unknown-linux-gnu -- -D warnings
cross clippy -p tiny-std --no-default-features --target aarch64-unknown-linux-gnu --tests -- -D warnings
cross clippy -p tiny-std --target aarch64-unknown-linux-gnu -- -D warnings
cross clippy -p tiny-std --target aarch64-unknown-linux-gnu --tests -- -D warnings
cross clippy -p tiny-std --target aarch64-unknown-linux-gnu --features executable -- -D warnings
