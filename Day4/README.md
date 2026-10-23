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

下面所有命令都是**手动执行**的，没有脚本。

---

## 1. 先有内核 `Image`

内核不是凭空来的，是 `make` 把 `linux-6.12.107` 源码编译出来的。

```bash
cd ~/riscv/linux-6.12.107

export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.83.0

make ARCH=riscv LLVM=1 defconfig
./scripts/config -e RUST
make ARCH=riscv LLVM=1 olddefconfig
make ARCH=riscv LLVM=1 -j"$(nproc)"
```

每行在干什么：

| 命令 | 作用 |
|---|---|
| `export PATH=...` | 让 `rustc`、`bindgen` 能被找到 |
| `export RUSTUP_TOOLCHAIN=1.83.0` | 固定用 1.83 的 Rust，和内核匹配 |
| `make ARCH=riscv LLVM=1 defconfig` | 生成 RISC-V 的默认配置 `.config` |
| `./scripts/config -e RUST` | 在 `.config` 里打开 `CONFIG_RUST=y` |
| `make ARCH=riscv LLVM=1 olddefconfig` | 把配置补全、解析依赖 |
| `make ARCH=riscv LLVM=1 -j$(nproc)` | 真正编译，`-j` 是并行任务数 |

`make` **不是编译器**，它是构建调度器：读 Makefile，决定先编哪个 `.c`、后链哪个 `.o`，然后调用 `clang`、`ld.lld`、`objcopy`。
你看到的 `CC` / `LD` / `OBJCOPY` 就是它在调这些工具。

编完得到：

```
arch/riscv/boot/Image
```

`Image` 是纯二进制（去掉了 ELF 外壳），里面是 RISC-V 指令。

---

## 2. 编译 `init`

```bash
rustup target add riscv64gc-unknown-linux-gnu --toolchain 1.83.0

rustc --target riscv64gc-unknown-linux-gnu -O \
  -C panic=abort \
  -C link-arg=-nostartfiles \
  -C link-arg=-static \
  -C link-arg=-no-pie \
  -C linker=riscv64-linux-gnu-gcc \
  -o init init.rs
```

`rustup target add` 是给 1.83 工具链补一个 RISC-V 目标的 `core` 库；没有它，`rustc` 找不到 `core`。

`rustc` 是编译器驱动：前端做宏展开/类型检查，LLVM 后端生成 RISC-V 机器码，最后调用链接器。

每个参数：

| 参数 | 作用 |
|---|---|
| `--target riscv64gc-unknown-linux-gnu` | 生成 RISC-V 64 位 Linux ABI 的代码 |
| `-O` | 优化（等价 `-C opt-level=2`） |
| `-C panic=abort` | panic 直接退出，不生成 unwinding 表 |
| `-C link-arg=-nostartfiles` | 不链接 `crt1.o/crti.o/crtn.o`，入口是自己的 `_start` |
| `-C link-arg=-static` | 静态链接，不依赖 `ld.so` |
| `-C link-arg=-no-pie` | 生成固定地址的 `EXEC`，不是 PIE |
| `-C linker=riscv64-linux-gnu-gcc` | 用 RISC-V 的交叉 gcc 当链接器 |
| `-o init init.rs` | 输入 `init.rs`，输出 `init` |

`-C link-arg=...` 的意思是“把后面的参数原样传给链接器”。

---

## 3. 用 readelf / objdump 研究 `init`

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

---

## 4. 打包 initramfs

```bash
rm -rf /tmp/rdinit
mkdir -p /tmp/rdinit
cp init /tmp/rdinit/init

(cd /tmp/rdinit && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9) \
  > /tmp/rdinit.cpio.gz
```

逐段：

| 命令 | 作用 |
|---|---|
| `cp init /tmp/rdinit/init` | 把 `init` 放到一个干净目录的根 |
| `cd /tmp/rdinit` | 一定要在归档根目录里执行 `find` |
| `find . -print0` | 列出当前目录所有文件，用 NUL 分隔 |
| `cpio --null -o -H newc` | 按 `newc` 格式打成 cpio 归档 |
| `gzip -9` | 压缩，内核 `CONFIG_RD_GZIP=y` 会解压 |
| `> /tmp/rdinit.cpio.gz` | 输出到文件 |

检查：

```bash
zcat /tmp/rdinit.cpio.gz | cpio -t
# .
# init
```

归档里只有 `init` 一个文件，没有 busybox、没有 libc。

---

## 5. 启动 QEMU

```bash
~/opt/qemu-7.0.0/bin/qemu-system-riscv64 \
  -machine virt -m 1G -smp 2 -nographic \
  -kernel ~/riscv/linux-6.12.107/arch/riscv/boot/Image \
  -initrd /tmp/rdinit.cpio.gz \
  -append "console=ttyS0"
```

| 参数 | 作用 |
|---|---|
| `-machine virt` | 模拟一块标准的 RISC-V 虚拟主板 |
| `-m 1G` | 虚拟内存 1 GB |
| `-smp 2` | 2 个虚拟 CPU |
| `-nographic` | 不开图形窗口，串口接终端 |
| `-kernel Image` | 加载 RISC-V 内核 |
| `-initrd rdinit.cpio.gz` | 把 initramfs 交给内核 |
| `-append "console=ttyS0"` | 内核命令行，日志走串口 |

QEMU **不是编译器**，它是一台用 x86 指令写成的虚拟 RISC-V 电脑：加载 `Image`，模拟 CPU 执行 RISC-V 指令。

预期：

```
[    1.xxx] Run /init as init process
[    1.xxx] hello riscv
[    1.xxx] reboot: Power down
```

---

## 6. 整条链

```
make（x86 宿主）  →  RISC-V 内核 Image
rustc（x86 宿主） →  RISC-V 用户程序 init
cpio + gzip       →  initramfs
QEMU（x86 宿主）  →  虚拟 RISC-V 机器
                      OpenSBI → Linux → /init → write/reboot
```

---

## 7. 下一步（P2）

把 Day3 的 463/464/465 和计数钩子移植到 RISC-V：

* `scripts/syscall.tbl` 加表项；
* `arch/riscv/kernel/traps.c` 的 `do_trap_ecall_u()` 加计数；
* `kernel/syscall_stats.c`、`rust_syscalls.rs` 复用；
* 用 `nm vmlinux` 验证符号。
