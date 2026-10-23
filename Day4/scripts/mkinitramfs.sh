#!/usr/bin/env bash
# 把 init 打包成 initramfs.cpio.gz（只有一个 /init）。
set -euo pipefail
cd "$(dirname "$0")/.."

[ -x init ] || { echo "缺少 init，先运行 scripts/build_init.sh" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp init "$tmp/init"
chmod +x "$tmp/init"

(cd "$tmp" && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9) \
	> initramfs.cpio.gz

echo "OK: $PWD/initramfs.cpio.gz"
