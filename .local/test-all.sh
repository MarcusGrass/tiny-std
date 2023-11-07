#!/bin/sh
set -ex
/bin/sh .local/rusl-test.sh
/bin/sh .local/tiny-std-test.sh
cargo test -p tiny-cli