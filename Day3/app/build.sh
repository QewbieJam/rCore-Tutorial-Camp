#!/usr/bin/env bash
# 编译三个 no_std 裸机程序，输出到本目录。
set -euo pipefail
cd "$(dirname "$0")"

for a in app_a app_b reset; do
	rustc --edition 2021 -O \
		-C panic=abort \
		-C relocation-model=static \
		-C link-arg=-nostartfiles \
		-C link-arg=-static \
		-C link-arg=-no-pie \
		-o "$a" "$a.rs"
	echo "built $a"
done
