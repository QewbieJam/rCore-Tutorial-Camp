# 在 Milk-V Jupiter v1.1 上启动

目标板：**Milk-V Jupiter v1.1**，SpacemiT K1/M1，RV64GCVB，Sv39。
板上固件链通常是：

```text
BootROM -> FSBL -> OpenSBI -> U-Boot -> jupiter-kernel
```

内核作为 S-mode payload 运行，不替换 FSBL/OpenSBI/U-Boot。

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

## 2. 硬件连接

- 调试串口：板载调试 UART（UART0，`0xd4017000`），115200 8N1，无流控。
- 供电：按板子说明使用 USB-C。
- 启动介质：可启动的 SD 卡（板上有 U-Boot 时也可以用 TFTP/USB）。

内核默认只管理前 2 GiB RAM，并且页帧分配从内核镜像结束地址之后开始。
因此低 512 MiB 会保留给 OpenSBI/U-Boot，牺牲容量换取安全。

## 3. 把内核放到板上

最简单：复制到 SD 卡的 FAT 分区。

```bash
cp target/riscv64gc-unknown-none-elf/release/jupiter-kernel /media/$USER/boot/
sync
```

如果 U-Boot 支持网络：

```text
setenv serverip 192.168.1.10
setenv ipaddr 192.168.1.20
tftp ${loadaddr} jupiter-kernel
```

## 4. U-Boot 命令

推荐把 ELF 文件加载到一块不与链接地址重叠的 RAM，再让 `bootelf` 展开：

```text
setenv kernel_addr_r 0x20000000
setenv elf_load_addr 0x30000000
load mmc 0:1 ${elf_load_addr} /jupiter-kernel
bootelf ${elf_load_addr}
```

`bootelf` 会按 ELF program header 把各段放到 `0x2000_0000` 起的链接地址。
之所以先把 ELF 文件本身放在 `0x3000_0000`，是为了避免“文件缓冲”和“目标段”
重叠导致展开时覆盖 program header。

如果 U-Boot 没有 `bootelf`，可以转成裸二进制后 `go`：

```bash
riscv64-unknown-elf-objcopy -O binary \
  target/riscv64gc-unknown-none-elf/release/jupiter-kernel \
  jupiter-kernel.bin
```

```text
load mmc 0:1 0x20000000 /jupiter-kernel.bin
go 0x20000000
```

裸二进制方式下 `a0/a1` 不再保证是 hartid/DTB；当前内核只打印这两个值，
不解析 DTB，所以不影响启动。后续接入 DTB 解析时需要改用 `bootelf` 或
由 U-Boot 显式传入 DTB。

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

### 没有任何输出

- 确认串口是 UART0、115200 8N1、TX/RX 没有接反。
- 确认 U-Boot 确实跳转到了 `0x2000_0000`。
- 用 `md 0x20000000 4` 检查内核镜像是否加载成功。
- 先用 QEMU 验证同一份代码逻辑：`make run`。

### 输出乱码

- 波特率不对，或 U-Boot 把 UART 配置成了别的时钟/格式。
- UART0 可能已被其他固件占用；确认使用的是调试串口而不是 40-pin 上的
  UART。

### 启动后立刻重启/卡死

- 确认构建的是 `BOARD=jupiter`，ELF 入口为 `0x20000000`。
- 确认加载地址没有覆盖 U-Boot 自身；先尝试 `elf_load_addr=0x30000000`。
- K1 的 DRAM 起始和保留区可能随固件版本变化；如果页帧分配覆盖了
  OpenSBI/U-Boot 区域，会在分配页表后崩溃。此时需要根据 DTB 的
  `memory` / `reserved-memory` 调整 `board/jupiter.rs`。
- 当前只映射前 2 GiB；4/8/16 GiB 板子只使用低 2 GiB。

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
