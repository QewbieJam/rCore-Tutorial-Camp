# jupiter-mk: 从 SD 卡加载并启动内核（U-Boot 脚本）
echo "=== jupiter-mk boot script ==="
setenv kernel_addr_r 0x30000000
for p in 1 2 3 4; do
    if load mmc 0:${p} ${kernel_addr_r} /jupiter-kernel; then
        echo "loaded /jupiter-kernel from mmc 0:${p} to ${kernel_addr_r}"
        bootelf ${kernel_addr_r}
    fi
done
echo "ERROR: /jupiter-kernel not found on mmc 0:1-4"
