#!/bin/sh
set -e
cd test-runners
cd small-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r -r --target aarch64-unknown-linux-gnu -- dummy_arg

cd ..
cd alloc-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r -r --target aarch64-unknown-linux-gnu -- dummy_arg
echo "Test runners completed successfully"
