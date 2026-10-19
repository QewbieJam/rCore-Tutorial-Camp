#!/usr/bin/env bash
# 启动 QEMU 测试模块。
#
#   KDIR=/path/to/linux ./scripts/run_qemu.sh [initramfs.cpio.gz]
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
: "${KDIR:=$HOME/linux-6.12.107}"
initrd="${1:-$root/initramfs.cpio.gz}"

[ -f "$KDIR/arch/x86/boot/bzImage" ] || { echo "缺少 $KDIR/arch/x86/boot/bzImage" >&2; exit 1; }
[ -f "$initrd" ] || { echo "缺少 $initrd，先运行 ./scripts/mkinitramfs.sh" >&2; exit 1; }

exec qemu-system-x86_64 \
	-m 1G -smp 2 -accel tcg,thread=multi \
	-kernel "$KDIR/arch/x86/boot/bzImage" \
	-initrd "$initrd" \
	-append "console=ttyS0 panic=-1" \
	-nographic -no-reboot
