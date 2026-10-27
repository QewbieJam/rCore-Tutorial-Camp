#!/usr/bin/env bash
# build.sh —— 一条命令走完：生成汇编 → 汇编链接 → QEMU 跑
# 用法:
#   ./build.sh hello.s       直接汇编一个手写 .s（验证脚手架）
#   ./build.sh examples/x.mc 跑你的编译器（.mc → .s → .elf → qemu）
set -euo pipefail
cd "$(dirname "$0")"

GCC=/bin/riscv64elf-gcc/bin/riscv64-unknown-elf-gcc
QEMU=/home/era/opt/qemu-7.0.0/bin/qemu-system-riscv64

INPUT=${1:-examples/hello.mc}

# 1) .mc 走编译器；.s 直接进汇编
case "$INPUT" in
  *.mc)
      echo "[1/3] minicc: $INPUT -> out.s" >&2
      cargo run --quiet --release -- "$INPUT" > out.s
      ASM=out.s
      ;;
  *)
      ASM="$INPUT"
      ;;
esac

# 2) 汇编 + 链接（runtime.S 提供 _start/print，linker.ld 定地址）
echo "[2/3] gcc: $ASM + runtime.S -> out.elf" >&2
$GCC -march=rv64imac -mabi=lp64 -mcmodel=medany \
     -nostdlib -T linker.ld \
     -o out.elf runtime.S "$ASM"

# 3) 跑到程序自己关机（SBI SRST），超时 10s 兜底
echo "[3/3] qemu" >&2
timeout 10 "$QEMU" -machine virt -bios default -kernel out.elf -nographic
echo "[exit=$?]" >&2
