#!/bin/sh
# 把 jupiter-kernel 和 boot.scr 复制到 SD 卡的 boot 分区。
#
# 用法：./scripts/install-to-sd.sh /mnt/jupiter-boot
set -e

MOUNT=${1:-}
if [ -z "$MOUNT" ]; then
    echo "用法: $0 <boot 分区挂载点>" >&2
    echo "例:   $0 /mnt/jupiter-boot" >&2
    exit 1
fi
if [ ! -d "$MOUNT" ]; then
    echo "错误: $MOUNT 不是目录（先 mount SD 卡的 boot 分区）" >&2
    exit 1
fi

DIR=$(cd "$(dirname "$0")/.." && pwd)
KERNEL="$DIR/sd-boot/jupiter-kernel"
SCRIPT="$DIR/sd-boot/boot.scr"

if [ ! -f "$KERNEL" ]; then
    echo "错误: 找不到 $KERNEL" >&2
    echo "先执行: make sd-bundle BOARD=jupiter" >&2
    exit 1
fi

TS=$(date +%Y%m%d-%H%M%S)
for f in jupiter-kernel boot.scr; do
    if [ -e "$MOUNT/$f" ]; then
        echo "备份 $MOUNT/$f -> $MOUNT/$f.bak.$TS"
        cp -p "$MOUNT/$f" "$MOUNT/$f.bak.$TS"
    fi
done

echo "复制 jupiter-kernel -> $MOUNT/"
cp -p "$KERNEL" "$MOUNT/"
if [ -f "$SCRIPT" ]; then
    echo "复制 boot.scr -> $MOUNT/"
    cp -p "$SCRIPT" "$MOUNT/"
else
    echo "警告: 没有 boot.scr，只能手动在 U-Boot 里 load + bootelf" >&2
fi

sync
echo "完成。现在 umount SD 卡，插到 Jupiter 上，串口 115200 观察。"
