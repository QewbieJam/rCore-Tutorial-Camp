# Day4：RISC-V 最小启动（P1）

## 目标

在 QEMU 的 RISC-V 虚拟机上跑一个自己写的 Rust `init`：

1. 打印 `hello riscv`；
2. 调 `reboot` 关机。

这一步**不涉及新系统调用**，只验证四件事：

* RISC-V 内核能编出来、能启动；
* Rust 能交叉编译出 RISC-V 的静态 ELF；
* initramfs 能被内核解压并执行 `/init`；
* 用户态 `ecall` 能真正打到内核。

## 文件

```
Day4/
├── init.rs
├── scripts/
│   ├── build_init.sh
│   ├── mkinitramfs.sh
│   └── run_qemu.sh
└── README.md
```

`init.rs` 关键部分：

```rust
#![no_std]
#![no_main]

global_asm!(
    ".global _start",
    "_start:",
    "mv s0, zero",
    "andi sp, sp, -16",
    "call app_main",
    "li a7, 93",
    "ecall",
);
```

## 编译

```bash
rustup target add riscv64gc-unknown-linux-gnu --toolchain 1.83.0
./scripts/build_init.sh
```

脚本等价于：

```bash
rustc --target riscv64gc-unknown-linux-gnu -O \
  -C panic=abort \
  -C link-arg=-nostartfiles \
  -C link-arg=-static \
  -C link-arg=-no-pie \
  -C linker=riscv64-linux-gnu-gcc \
  -o init init.rs
```

## 用 readelf / objdump 研究它

宿主的 `readelf` 不一定认 RISC-V，用交叉工具或 LLVM：

```bash
llvm-readelf -h init
riscv64-linux-gnu-readelf -S init
riscv64-linux-gnu-readelf -l init
riscv64-linux-gnu-nm -n init
llvm-objdump -d --triple=riscv64 init
```

### ELF 头

```
Type:            EXEC (Executable file)
Machine:         RISC-V
Entry point:     0x10144
Flags:           0x5, RVC, double-float ABI
```

* `EXEC`：固定地址可执行，不是 PIE；
* `Entry point = 0x10144`：就是 `_start` 的地址；
* `RVC`：使用了压缩指令（16 位）；
* `double-float ABI`：lp64d。

### 节

```
.text        0x10144  大小 0x52   AX
.rodata      0x10196  大小 0x0c   A
```

* `.text` 只有 82 字节：入口 + `app_main`；
* `.rodata` 12 字节：就是 `"hello riscv\n"`。

### 程序头

```
LOAD  offset 0x0  vaddr 0x10000  filesz 0x1a2  R E  align 0x1000
```

只有一个 `LOAD`，没有 `INTERP`，说明它是**静态**的，不依赖 `ld.so`。

### 符号

```
0000000000010144 T _start
000000000001015a T app_main
```

`T` 表示在 `.text` 里定义。`_start` 在 `0x10144`，`app_main` 在 `0x1015a`。

### 反汇编（关键部分）

```
0000000000010144 <_start>:
   10144: 4401          li   s0, 0x0
   10146: ff017113      andi sp, sp, -0x10
   1014a: 00000097      auipc ra, 0x0
   1014e: 010080e7      jalr 0x10(ra) <app_main>
   10152: 05d00893      li   a7, 0x5d          # 93 = exit
   10156: 00000073      ecall

000000000001015a <app_main>:
   1015a: 00000517      auipc a0, 0x0
   1015e: 03c50593      addi a1, a0, 0x3c       # a1 = 字符串地址
   10162: 4505          li   a0, 0x1            # fd = 1
   10164: 4631          li   a2, 0xc            # len = 12
   10166: 04000893      li   a7, 0x40           # 64 = write
   1016a: 00000073      ecall
   1016e: 28122537      lui  a0, 0x28122
   10172: 9695059b      addiw a1, a0, -0x697    # magic2
   10176: 43220537      lui  a0, 0x43220
   1017a: edc5061b      addiw a2, a0, -0x124    # cmd
   1017e: 7f70f537      lui  a0, 0x7f70f
   10182: 0506          slli a0, a0, 0x1
   10184: ead50513      addi a0, a0, -0x153    # magic1
   10188: 08e00893      li   a7, 0x8e           # 142 = reboot
   1018c: 4681          li   a3, 0x0
   1018e: 00000073      ecall
   10192: 4501          li   a0, 0x0
   10194: 8082          ret
```

对照源码：

* `_start` 就是那 6 条指令；
* `write` 用 `auipc` 算字符串地址，`a0=1, a2=12, a7=64`；
* `reboot` 的四个参数分别放 `a0`（magic1）、`a1`（magic2）、`a2`（cmd）、`a3`（arg），`a7=142`；
* 最后 `li a0,0; ret` 只是 `app_main` 返回 0。

## 打包 initramfs

```bash
./scripts/mkinitramfs.sh
```

归档里只有 `init` 一个文件，没有 busybox、没有 libc。

## 启动

```bash
./scripts/run_qemu.sh
# 或
QEMU=~/opt/qemu-7.0.0/bin/qemu-system-riscv64 ./scripts/run_qemu.sh
```

预期：

```
[    1.xxx] Run /init as init process
[    1.xxx] hello riscv
[    1.xxx] reboot: Power down
```

## 这一步在整条链里的位置

```
make（x86 宿主）  →  RISC-V 内核 Image
rustc（x86 宿主） →  RISC-V 用户程序 init
cpio+gzip         →  initramfs
QEMU（x86 宿主）  →  虚拟 RISC-V 机器
                      OpenSBI → Linux → /init → write/reboot
```

## 下一步（P2）

把 Day3 的 463/464/465 和计数钩子移植到 RISC-V：

* `scripts/syscall.tbl` 加表项；
* `arch/riscv/kernel/traps.c` 的 `do_trap_ecall_u()` 加计数；
* `kernel/syscall_stats.c`、`rust_syscalls.rs` 复用；
* 用 `nm vmlinux` 验证符号。
