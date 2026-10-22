#!/usr/bin/env bash
# 启动 QEMU 跑 Day3。
#
#   KDIR=/path/to/linux ./scripts/run_qemu.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
: "${KDIR:=$HOME/linux-6.12.107}"

[ -f "$KDIR/arch/x86/boot/bzImage" ] || { echo "缺少 $KDIR/arch/x86/boot/bzImage" >&2; exit 1; }
[ -f "$root/initramfs.cpio.gz" ] || { echo "先运行 scripts/mkinitramfs.sh" >&2; exit 1; }

exec qemu-system-x86_64 \
	-m 1G -smp 2 -accel tcg,thread=multi \
	-kernel "$KDIR/arch/x86/boot/bzImage" \
	-initrd "$root/initramfs.cpio.gz" \
	-append "console=ttyS0 panic=-1" \
	-nographic -no-reboot
