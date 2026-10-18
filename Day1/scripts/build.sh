#!/usr/bin/env bash
# 编译树外 Rust 内核模块。
#
#   KDIR=/path/to/linux ./scripts/build.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
: "${KDIR:=$HOME/linux-6.12.107}"

if [ ! -d "$KDIR/rust" ]; then
	echo "错误: $KDIR/rust 不存在，KDIR 不是完整编译过的 Rust 内核树" >&2
	exit 1
fi

make -C "$KDIR" M="$root" LLVM=1 modules
echo "OK: $root/rust_hello.ko"
