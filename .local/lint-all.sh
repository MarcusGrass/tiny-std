#!/bin/sh
set -ex
/bin/sh .local/rusl-lint.sh
/bin/sh .local/tiny-std-lint.sh
