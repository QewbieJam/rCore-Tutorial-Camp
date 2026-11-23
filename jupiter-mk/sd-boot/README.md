# 上板启动包（sd-boot/）

这个目录是给 **Milk-V Jupiter v1.1** 用的 SD 卡启动材料。

```
sd-boot/
├── jupiter-kernel   ← 编译产物（ELF，入口 0x2000_0000），gitignore
├── boot.cmd         ← U-Boot 脚本源码
├── boot.scr         ← mkimage 编译后的脚本，gitignore
└── README.md        ← 本文件
```

## 0. 前置条件

| 项 | 要求 |
|---|---|
| SD 卡 | 16GB+，**已用 Milk-V 官方 Jupiter 镜像打底**（含 FSBL/OpenSBI/U-Boot） |
| 串口 | 3.3V USB-TTL，接 GND/TX/RX（**不接红线**），115200 8N1 |
| 电脑 | 能挂载 SD 卡 boot 分区（FAT32 或 ext4 都行） |

> 纯空白 FAT32 卡启动不了：K1 的 BootROM 不认 FAT 里的 U-Boot，必须先用官方镜像把启动链做好。

## 1. 构建

```bash
cd ~/rCore-Tutorial-Camp/jupiter-mk
make sd-bundle BOARD=jupiter
```

`sd-bundle` 会：
1. 编译内核（`BOARD=jupiter`）
2. 复制 ELF 到 `sd-boot/jupiter-kernel`
3. 用 `mkimage` 生成 `sd-boot/boot.scr`

检查入口地址：

```bash
riscv64-unknown-elf-readelf -h sd-boot/jupiter-kernel | grep Entry
# Entry point address: 0x20000000
```

## 2. 拷到 SD 卡

```bash
# ① 确认 SD 卡设备（千万别选错盘）
lsblk -o NAME,SIZE,TYPE,LABEL,MOUNTPOINT

# ② 找到 boot 分区并挂载（假设是 /dev/sdb1，以实际为准）
sudo mkdir -p /mnt/jupiter-boot
sudo mount /dev/sdb1 /mnt/jupiter-boot
ls /mnt/jupiter-boot

# ③ 复制（脚本会备份同名文件）
./scripts/install-to-sd.sh /mnt/jupiter-boot

# ④ 卸载
sync
sudo umount /mnt/jupiter-boot
```

## 3. 启动

### 3.1 自动（boot.scr 生效时）

有些 U-Boot 的 distro boot 会自动扫描并执行 `boot.scr`。插卡上电，串口里应该直接看到：

```
=== jupiter-mk boot script ===
loaded /jupiter-kernel from mmc 0:1 to 0x30000000
==============================================
  jupiter-mk / RISC-V microkernel
  board : milk-v-jupiter-v1.1 (spacemit k1)
  hart  : 0
  dtb   : 0x...
```

### 3.2 手动（推荐第一次用）

上电时在串口看到：

```
Hit any key to stop autoboot:  3
```

按任意键停住，然后：

```
mmc list
mmc dev 0
ls mmc 0:1 /
```

确认 `/jupiter-kernel` 在哪个分区（可能是 `0:1` ~ `0:6`）。
> **注意**：Milk-V 官方 Jupiter 镜像的 bootfs 通常是第 5 个分区，即 `mmc 0:5`（ext4）。

然后：

```
setenv kernel_addr_r 0x30000000
load mmc 0:5 ${kernel_addr_r} /jupiter-kernel
bootelf ${kernel_addr_r}
```

或者直接跑脚本：

```
load mmc 0:5 ${kernel_addr_r} /boot.scr
source ${kernel_addr_r}
```

## 4. 排错

| 现象 | 可能原因 | 处理 |
|---|---|---|
| `load mmc` 报 `Failed to load` | 分区不对 / 文件系统不支持 | `ls mmc 0:2 /`、`ls mmc 0:3 /` 换着试；ext4 用 `ext4load`，FAT 用 `fatload` |
| 加载成功但无输出 | 串口接错 / UART 基址不对 | 确认 GND/TX/RX、115200；Jupiter UART0 = `0xd4017000` |
| 输出乱码 | 波特率或电平不对 | 115200 8N1，3.3V 电平 |
| 卡在 U-Boot | `bootelf` 没执行 | 手动执行 `bootelf ${kernel_addr_r}` |
| 一直重启 | 内核异常 | 看串口最后一行；确认 `BOARD=jupiter` 构建（入口 `0x20000000`） |
| 进 Linux 了 | 自动启动了 eMMC/其他分区 | 停 autoboot；或临时 `setenv bootcmd ''`（本次生效） |

## 5. 注意事项

- **不要覆盖 SD 卡上的原始文件**（FSBL/U-Boot/Linux），只往 boot 分区根目录加两个文件。
- 内核链接地址是 `0x2000_0000`，U-Boot 把它 load 到 `0x3000_0000` 再 `bootelf` 是安全的（两者不重叠）。
- 如果想让板子默认永不进 Linux，可以在 U-Boot 里 `setenv bootcmd 'load mmc 0:1 0x30000000 /jupiter-kernel; bootelf 0x30000000'` 然后 `saveenv`（**这会改板子的持久配置**，建议先用一次性的手动命令）。
