# 在 Milk-V Jupiter v1.1 上启动

目标板：**Milk-V Jupiter v1.1**，SpacemiT K1/M1，RV64GCVB，Sv39。
板上固件链通常是：

```text
BootROM -> FSBL -> OpenSBI -> U-Boot -> jupiter-kernel
```

内核作为 S-mode payload 运行，不替换 FSBL/OpenSBI/U-Boot。

> 不需要显示器。Jupiter 有 3.3V 调试 UART，接 USB-TTL 后在电脑上看串口即可；
> U-Boot、内核日志和 shell 都走这个串口。

## 1. 构建

```bash
cd ~/rCore-Tutorial-Camp/jupiter-mk
make kernel BOARD=jupiter
```

产物：

```text
target/riscv64gc-unknown-none-elf/release/jupiter-kernel
```

链接地址是 **0x2000_0000**（见 `kernel/linker-jupiter.ld`）。可以检查：

```bash
riscv64-unknown-elf-readelf -h target/riscv64gc-unknown-none-elf/release/jupiter-kernel
# Entry point address: 0x20000000
```

注意：`BOARD=qemu` 和 `BOARD=jupiter` 输出到同一个 Cargo target 路径。
上板前确认最后一次构建带 `BOARD=jupiter`。

## 2. 硬件与串口

### 串口

- Jupiter 的调试 UART 是 **3.3V 电平**，需要 USB-TTL 串口线。
- **不要接 USB-TTL 的红色电源线**，只接 GND / TX / RX。
- 参数：**115200 8N1，无流控**。
- 电脑侧设备通常是 `/dev/ttyUSB0`（CH340/CP210x 等）。

Linux 下：

```bash
sudo usermod -aG dialout $USER   # 第一次使用，重新登录后生效
picocom -b 115200 /dev/ttyUSB0
# 或
screen /dev/ttyUSB0 115200
# 或
tio -b 115200 /dev/ttyUSB0
```

退出 `picocom`：`Ctrl-A` 然后 `Ctrl-X`。

### 启动介质

Jupiter 支持从 SD 卡、eMMC、SSD 启动，优先级是：

```text
SD 卡 > SSD > eMMC
```

板上 SPI Flash 可以放 U-Boot，但容量小，通常只放启动程序。
**一张完全空白、只有 FAT32 分区的 SD 卡不能启动**：BootROM 不会去 FAT 分区里
找 U-Boot，而是按厂商定义的偏移/格式读取 FSBL。因此第一次上板，最简单的是
用官方镜像把 SD 卡做成“带完整启动链”的卡，再把我们的内核放进去。

## 3. SD 卡准备（第一次必看）

### 推荐路线：官方 SD 镜像打底

Milk-V 官方提供 SD 卡专用镜像（后缀通常是 `.img.zip`），里面已经包含：

```text
分区表 + FSBL + OpenSBI + U-Boot + boot 分区 + Linux rootfs
```

我们只借用它的启动链，Linux 可以完全不启动。

步骤：

1. 准备一张 **16 GB 或更大的 SD 卡**（8 GB 可能不够官方镜像）。
   卡里的数据会被清空，先备份。

2. 从 Milk-V 资源页下载 Jupiter 的 SD 镜像（`.img.zip`）。
   如果需要，先解压得到 `.img`：

   ```bash
   unzip bianbu-*.img.zip
   # 或
   xz -d xxx.img.xz
   ```

3. 找到 SD 卡设备。**这一步一定要确认设备名，写错会毁掉电脑硬盘**：

   ```bash
   lsblk -o NAME,SIZE,TYPE,MODEL,MOUNTPOINT
   ```

   假设 SD 卡是 `/dev/sdb`（不是 `/dev/sdb1`，要写整盘）。

4. 用 `dd` 或 balenaEtcher 烧录：

   ```bash
   sudo dd if=bianbu-*.img of=/dev/sdb bs=4M status=progress conv=fsync
   sudo sync
   ```

   或者用 balenaEtcher 选择 `.img` / `.img.zip` 烧录，更不容易选错盘。

5. 烧完后，Linux 可能自动挂载 boot 分区。用 `lsblk -f` 看分区：

   ```bash
   lsblk -f
   ```

   典型情况下会有类似：

   ```text
   sdb
   ├─sdb1  ext4   bootfs
   ├─sdb2  ext4   rootfs
   ```

   也可能是 `vfat` 的 boot 分区。**不用纠结文件系统类型**，能挂载就行。

6. 把我们的内核复制到 boot 分区根目录：

   ```bash
   sudo mkdir -p /mnt/jupiter-boot
   sudo mount /dev/sdb1 /mnt/jupiter-boot
   sudo cp target/riscv64gc-unknown-none-elf/release/jupiter-kernel /mnt/jupiter-boot/
   sync
   sudo umount /mnt/jupiter-boot
   ```

   如果 `sdb1` 不是 boot 分区，就试 `sdb2`，或者看 `lsblk -f` 里的标签。

7. 把 SD 卡插回 Jupiter，接串口，上电。

### 不想保留 Linux 可以吗？

可以，但**不建议第一次就做“纯 U-Boot + 内核”卡**。原因是 K1 的 BootROM
需要 FSBL 在特定位置，手动分区 + 放一个 U-Boot 文件通常启动不了。正确做法是：

- 先用官方镜像打底；
- 确认 U-Boot 和串口都正常；
- 之后再考虑裁剪 rootfs，或者用 `titanflasher` / `fastboot` 刷自己的
  FSBL/OpenSBI/U-Boot。

对当前阶段来说，保留 rootfs 不影响我们，只是占空间。

### 如果板子已经有 U-Boot（SPI/eMMC）

有些板子会把 U-Boot 放在 SPI Flash，并从 eMMC/SSD 启动系统。注意 Milk-V 的
说明是“SD 卡启动不经过 SPI Flash”，所以**不要指望 SPI 里的 U-Boot 去加载
一张只有内核文件的空白 SD 卡**。

如果你确实是从 eMMC/SSD 启动、并且 U-Boot 已经在 SPI 里，可以进入 U-Boot
后手动从 SD/U 盘/TFTP 加载内核。但这属于第二条路线，第一次上板仍然建议
用官方 SD 镜像打底，成功率高、变量少。

## 4. U-Boot 加载内核

上电后在串口里看日志，出现类似：

```text
Hit any key to stop autoboot:  3
```

时按任意键，进入 U-Boot 命令行。如果直接进了 Linux，重新上电，在倒计时期间
连续按键；必要时先设置：

```text
setenv bootdelay 3
saveenv
```

### 4.1 从 SD 卡加载

先确认 MMC 设备和分区：

```text
mmc list
mmc dev 0
ls mmc 0:1 /
ls mmc 0:1 /jupiter-kernel
```

如果 `0:1` 不是文件系统，试 `0:2`、`0:3`。找到内核后：

```text
setenv elf_load_addr 0x30000000
load mmc 0:1 ${elf_load_addr} /jupiter-kernel
bootelf ${elf_load_addr}
```

要点：

- 内核链接在 `0x2000_0000`，但 ELF 文件本身先加载到 `0x3000_0000`，
  避免 `bootelf` 展开时“文件缓冲区”和“目标段”重叠。
- `bootelf` 会按 ELF program header 把各段放到 `0x2000_0000` 起。
- 如果 `load` 不识别 ext4，可以试 `ext4load mmc 0:1 ...`。

### 4.2 没有 `bootelf` 时

先转成裸二进制：

```bash
riscv64-unknown-elf-objcopy -O binary \
  target/riscv64gc-unknown-none-elf/release/jupiter-kernel \
  jupiter-kernel.bin
```

把 `jupiter-kernel.bin` 复制到 SD 卡，然后：

```text
load mmc 0:1 0x20000000 /jupiter-kernel.bin
go 0x20000000
```

裸二进制方式下 `a0/a1` 不再保证是 hartid/DTB；当前内核只打印这两个值，
不解析 DTB，所以不影响启动。后续接入 DTB 解析时需要改用 `bootelf` 或
由 U-Boot 显式传入 DTB。

### 4.3 从 U 盘加载（不改 SD 卡分区）

如果不想动 SD 卡的 boot 分区，可以把 `jupiter-kernel` 放到一个 FAT32 U 盘：

```text
usb start
ls usb 0:1 /
load usb 0:1 0x30000000 /jupiter-kernel
bootelf 0x30000000
```

前提是 U-Boot 编译了 USB storage 支持。

### 4.4 从网络加载（TFTP）

网络只解决“反复改内核”的效率问题；**第一次仍然需要 SD/eMMC/SPI 里的
U-Boot 先跑起来**。

U-Boot 侧，能 DHCP 就最简单：

```text
dhcp
tftp 0x30000000 jupiter-kernel
bootelf 0x30000000
```

静态 IP：

```text
setenv ipaddr 192.168.1.20
setenv serverip 192.168.1.10
setenv netmask 255.255.255.0
ping ${serverip}
tftp 0x30000000 jupiter-kernel
bootelf 0x30000000
```

电脑侧准备 TFTP 服务：

```bash
sudo apt install tftpd-hpa
sudo systemctl status tftpd-hpa

# 默认目录一般是 /srv/tftp
sudo cp target/riscv64gc-unknown-none-elf/release/jupiter-kernel /srv/tftp/
sudo chmod 644 /srv/tftp/jupiter-kernel
```

注意防火墙放行 UDP 69；板子和电脑要在同一网段。
如果 `dhcp` 失败，检查网线、U-Boot 是否带 K1 网卡驱动，以及是否设置了
`ethaddr`。

## 5. 预期输出

```text
==============================================
  jupiter-mk / RISC-V microkernel
  board : milk-v-jupiter-v1.1 (spacemit k1)
  hart  : 0
  dtb   : 0x...
  ram   : 0x0..0x80000000
  uart  : 0xd4017000
  load  : 0x20000000
==============================================
[mm] kernel heap: 16384 KiB
[mm] frames: ...
[mm] Sv39 kernel identity map ...
...
[m3] entering user mode: console server + shell

[console] uart driver online
...
> 
```

看到 `>` 后可以输入 `help`、`hello`、`pid`、`mem`。

## 6. 排错

### 串口没有任何输出

- 确认用的是 3.3V 调试 UART，TX/RX 交叉连接，GND 共地。
- **不要接 USB-TTL 的红线（电源）**。
- 确认波特率 115200 8N1、无流控。
- 换一根 USB-TTL 线或换一个 USB 口；确认电脑识别出 `/dev/ttyUSB0`。
- 不插 SD 卡上电，看是否有 U-Boot 输出，以判断 U-Boot 在哪个介质上。

### 串口有 U-Boot，但加载内核失败

- `mmc list` 确认 SD 是 `mmc 0` 还是 `mmc 1`。
- `ls mmc 0:1 /` 确认分区号和文件系统。
- 确认文件名就是 `jupiter-kernel`，没有多余后缀。
- `load` 失败时试 `ext4load`；或者把内核放到 FAT32 U 盘，用 `usb start`。

### 内核加载后立刻重启/卡死

- 确认构建的是 `BOARD=jupiter`，ELF 入口为 `0x20000000`。
- 确认 `bootelf` 的加载地址是 `0x30000000`，不要和 `0x20000000` 重叠。
- K1 的 DRAM 起始和保留区可能随固件版本变化；如果页帧分配覆盖了
  OpenSBI/U-Boot 区域，会在分配页表后崩溃。此时需要根据 DTB 的
  `memory` / `reserved-memory` 调整 `board/jupiter.rs`。
- 当前只映射前 2 GiB；4/8/16 GiB 板子只使用低 2 GiB。
- 如果 `go` 裸二进制方式启动，确认 BSS 清零逻辑在 `entry.asm` 里
  （当前已实现）。

### 输出乱码

- 波特率不对，或 U-Boot 把 UART 配置成了别的时钟/格式。
- 确认使用的是调试 UART，而不是 40-pin 上引出的其他 UART。

### `bootelf` 报错

- 某些 U-Boot 版本的 `bootelf` 对 ELF program header 对齐较严格。
  可以直接用 `objcopy -O binary` + `go`。
- 也可以把内核改成 FIT image；这属于后续板级打包工作。

## 7. 当前未验证项

- 真实 K1 上的 UART 寄存器行为（8250 兼容性）需要实测。
- 定时器使用 SBI TIME，频率取 24 MHz；如果实际 timebase 不同，
  抢占频率会偏差，但不会影响功能。
- PLIC 尚未使用；console 采用轮询，不依赖 UART 中断。
- DTB 尚未解析；内存范围和 UART 地址目前是板级常量。
- 网络启动依赖 U-Boot 的 K1 网卡驱动，需要实板确认。
