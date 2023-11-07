#!/bin/sh
set -ex
cargo fmt --all --check
cd test-runners/alloc-st-main
cargo fmt --all --check
cd ../no-alloc-main
cargo fmt --all --check
cd ../test-lib
cargo fmt --all --check
cd ../threaded-main
cargo fmt --all --check
cd ../..
/bin/sh .local/rusl-lint.sh
/bin/sh .local/tiny-std-lint.sh
cargo clippy -p tiny-cli -- -D warnings
