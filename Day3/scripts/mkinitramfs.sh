#!/usr/bin/env bash
# 把 init、busybox、三个 Rust app 打包成 initramfs.cpio.gz。
#
#   BUSYBOX=/path/to/busybox ./scripts/mkinitramfs.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
busybox="${BUSYBOX:-/bin/busybox}"

[ -f "$busybox" ] || { echo "找不到 busybox: $busybox（可用 BUSYBOX=... 指定）" >&2; exit 1; }
for a in reset app_a app_b; do
	[ -x "$root/app/$a" ] || { echo "缺少 app/$a，先运行 app/build.sh" >&2; exit 1; }
done

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp"/{bin,proc,sys,dev}
cp "$root/initramfs/init" "$tmp/init"
chmod +x "$tmp/init"
cp "$busybox" "$tmp/bin/busybox"
chmod +x "$tmp/bin/busybox"

# 动态链接的 busybox 需要把依赖库也带进去。
if file "$busybox" | grep -q 'dynamically linked'; then
	ldd "$busybox" | awk '/=> \// {print $3} /^\// {print $1}' | while read -r lib; do
		[ -f "$lib" ] || continue
		mkdir -p "$tmp$(dirname "$lib")"
		cp "$lib" "$tmp$lib"
	done
fi

cp "$root/app/reset" "$root/app/app_a" "$root/app/app_b" "$tmp/"
chmod +x "$tmp"/reset "$tmp"/app_a "$tmp"/app_b

(cd "$tmp" && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9) \
	> "$root/initramfs.cpio.gz"

echo "OK: $root/initramfs.cpio.gz"
