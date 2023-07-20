#!/bin/sh
set -e
cd test-runners
cd no-alloc-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main debug"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main release"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=static' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main release static absolute"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=pie' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran small main release static pie"

cd ..
cd alloc-st-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main debug"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=static' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release static absolute"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=pie' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release static pie"

cd ..
cd threaded-main
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main debug"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=static' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release static absolute"
RUSTFLAGS='-C panic=abort -C link-arg=-nostartfiles -C target-feature=+crt-static -C relocation-model=pie' cargo r -r --target x86_64-unknown-linux-gnu -- dummy_arg
echo "Built and ran alloc main release static pie"
echo "x86_64 Test runners completed successfully"