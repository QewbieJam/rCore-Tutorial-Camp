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

# 先确认可写（SD 卡锁开关、只读挂载、vfat 错误都会导致写失败）
if ! touch "$MOUNT/.write_test" 2>/dev/null; then
    echo "错误: $MOUNT 是只读的，无法写入。可能原因：" >&2
    echo "  1) SD 卡侧面的写保护开关被锁住了（拨到 unlock）" >&2
    echo "  2) 挂载时带了 ro：sudo mount -o remount,rw $MOUNT" >&2
    echo "  3) vfat 文件系统有错误：sudo fsck.vfat -a /dev/<卡设备>" >&2
    exit 1
fi
rm -f "$MOUNT/.write_test"

TS=$(date +%Y%m%d-%H%M%S)
for f in jupiter-kernel boot.scr; do
    if [ -e "$MOUNT/$f" ]; then
        echo "备份 $MOUNT/$f -> $MOUNT/$f.bak.$TS"
        cp "$MOUNT/$f" "$MOUNT/$f.bak.$TS"
    fi
done

# 注意：vfat 不支持 Unix 权限/所有权，不能用 cp -p
echo "复制 jupiter-kernel -> $MOUNT/"
cp "$KERNEL" "$MOUNT/"
if [ -f "$SCRIPT" ]; then
    echo "复制 boot.scr -> $MOUNT/"
    cp "$SCRIPT" "$MOUNT/"
else
    echo "警告: 没有 boot.scr，只能手动在 U-Boot 里 load + bootelf" >&2
fi

sync
echo "完成。写入的文件："
ls -l "$MOUNT/jupiter-kernel" "$MOUNT/boot.scr" 2>/dev/null
echo "现在 umount SD 卡，插到 Jupiter 上，串口 115200 观察。"
