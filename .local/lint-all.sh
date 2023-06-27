#!/bin/sh
set -ex
cargo fmt --all --check
/bin/sh .local/rusl-lint.sh
/bin/sh .local/tiny-std-lint.sh
