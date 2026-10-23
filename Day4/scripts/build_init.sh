#!/usr/bin/env bash
# 编译 RISC-V 版 init（no_std，静态 ELF）。
set -euo pipefail
cd "$(dirname "$0")/.."

: "${RUSTUP_TOOLCHAIN:=1.83.0}"
export RUSTUP_TOOLCHAIN

rustc --target riscv64gc-unknown-linux-gnu -O \
	-C panic=abort \
	-C link-arg=-nostartfiles \
	-C link-arg=-static \
	-C link-arg=-no-pie \
	-C linker=riscv64-linux-gnu-gcc \
	-o init init.rs

file init
