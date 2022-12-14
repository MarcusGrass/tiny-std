#!/bin/sh
set -ex
/bin/sh .local/lint-all.sh
/bin/sh .local/test-all.sh
