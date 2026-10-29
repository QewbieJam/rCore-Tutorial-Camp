#!/usr/bin/env bash
# 全链路：add3.rs → add3.s → add3.elf → QEMU 运行
# 依赖: rustc(带 riscv64gc-unknown-none-elf target) / riscv64-unknown-elf-gcc / qemu-system-riscv64
set -euo pipefail
cd "$(dirname "$0")"

GCC=${GCC:-riscv64-unknown-elf-gcc}
QEMU=${QEMU:-qemu-system-riscv64}

echo "[1/4] rustc: add3.rs -> add3.s"
rustc --target riscv64gc-unknown-none-elf --crate-type lib \
      --emit asm -O -o add3.s add3.rs

echo "[2/4] 去掉 .attribute 行（老版 gcc 汇编器不认新 ISA 字符串）"
sed '/^[[:space:]]*\.attribute/d' add3.s > add3_gcc.s

echo "[3/4] 汇编 + 链接（runtime.S 提供 _start / print_int）"
$GCC -march=rv64imac -mabi=lp64 -mcmodel=medany \
     -nostdlib -T linker.ld -o add3.elf runtime.S add3_gcc.s

echo "[4/4] QEMU 运行"
timeout 10 $QEMU -machine virt -bios default -kernel add3.elf -nographic
echo
