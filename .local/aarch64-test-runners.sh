#!/bin/sh
set -e
cd test-runners
cd no-alloc-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=pie' cross b -r --target aarch64-unknown-linux-gnu
qemu-aarch64 target/aarch64-unknown-linux-gnu/release/no-alloc-main dummy_arg

cd ..
cd alloc-st-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=pie' cross b -r --target aarch64-unknown-linux-gnu
qemu-aarch64 target/aarch64-unknown-linux-gnu/release/alloc-st-main dummy_arg

cd ..
cd threaded-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=pie' cross b -r --target aarch64-unknown-linux-gnu
qemu-aarch64 target/aarch64-unknown-linux-gnu/release/threaded-main dummy_arg
echo "Aarch64 Test runners completed successfully"
