#!/bin/sh
set -e
cd test-runners
cd small-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=static' cross r -r --target aarch64-unknown-linux-gnu -- dummy_arg

cd ..
cd alloc-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=static' cross r -r --target aarch64-unknown-linux-gnu -- dummy_arg
echo "Aarch64 Test runners completed successfully"
