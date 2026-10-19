#!/usr/bin/env bash
# 把 busybox、动态库、init 和 syscall_stack.ko 打包成 initramfs。
#
#   ./scripts/mkinitramfs.sh [输出文件]
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/initramfs.cpio.gz}"

for f in /usr/bin/busybox /lib64/ld-linux-x86-64.so.2 \
         /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/libresolv.so.2; do
	[ -e "$f" ] || { echo "缺少 $f" >&2; exit 1; }
done

[ -f "$root/syscall_stack.ko" ] || { echo "先运行 ./scripts/build.sh" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp"/{bin,lib64,lib/x86_64-linux-gnu,proc,sys,dev}
cp /usr/bin/busybox "$tmp/bin/"
cp /lib64/ld-linux-x86-64.so.2 "$tmp/lib64/"
cp /lib/x86_64-linux-gnu/libc.so.6 \
   /lib/x86_64-linux-gnu/libresolv.so.2 "$tmp/lib/x86_64-linux-gnu/"
cp "$root/initramfs/init" "$tmp/init"
cp "$root/syscall_stack.ko" "$tmp/"
chmod +x "$tmp/init"

(cd "$tmp" && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9) > "$out"
echo "OK: $out"
