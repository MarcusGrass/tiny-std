#!/bin/sh
set -ex
cargo fmt --all
cd test-runners/alloc-st-main
cargo fmt --all
cd ../no-alloc-main
cargo fmt --all
cd ../test-lib
cargo fmt --all
cd ../threaded-main
cargo fmt --all
