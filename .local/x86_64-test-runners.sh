#!/bin/sh
set -e
cd test-runners
cd small-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -C target-feature=+crt-static -C relocation-model=pie -g' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main release static pie"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -g' cargo r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main debug"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -g' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main release"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -C target-feature=+crt-static -C relocation-model=static -g' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main release static absolute"


cd ..
cd alloc-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -g' cargo r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main debug"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -g' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -C target-feature=+crt-static -C relocation-model=static -g' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release static absolute"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C link-arg=-fuse-ld=mold -C target-feature=+crt-static -C relocation-model=pie -g' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release static pie"
echo "x86_64 Test runners completed successfully"