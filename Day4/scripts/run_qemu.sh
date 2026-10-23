#!/usr/bin/env bash
# 启动 QEMU RISC-V，跑 Day4 的 init。
#
#   KDIR=/path/to/riscv/linux ./scripts/run_qemu.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
: "${KDIR:=$HOME/riscv/linux-6.12.107}"
: "${QEMU:=qemu-system-riscv64}"

[ -f "$KDIR/arch/riscv/boot/Image" ] || { echo "缺少 $KDIR/arch/riscv/boot/Image" >&2; exit 1; }
[ -f "$root/initramfs.cpio.gz" ] || { echo "先运行 scripts/mkinitramfs.sh" >&2; exit 1; }

exec "$QEMU" \
	-machine virt -m 1G -smp 2 -nographic \
	-kernel "$KDIR/arch/riscv/boot/Image" \
	-initrd "$root/initramfs.cpio.gz" \
	-append "console=ttyS0"
