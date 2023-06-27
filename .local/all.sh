#!/bin/sh
set -ex
/bin/sh .local/lint-all.sh
/bin/sh .local/test-all.sh
/bin/sh .local/x86_64-test-runners.sh
/bin/sh .local/aarch64-test-runners.sh
