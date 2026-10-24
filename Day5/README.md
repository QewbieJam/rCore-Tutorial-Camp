# Day5：RISC-V 系统调用（P2a / P2b）

## P2a：把 Day3 的三个系统调用移植到 RISC-V

| 编号 | 名字 | 作用 |
|---|---|---|
| 463 | `get_taskinfo` | 当前 task 的 tid / pid / comm |
| 464 | `get_syscall_count` | 某个系统调用被调用次数 |
| 465 | `reset_syscall_counts` | 清零计数 |

和 x86 Day3 的区别：

| | x86 | RISC-V |
|---|---|---|
| syscall 表 | `arch/x86/entry/syscalls/syscall_64.tbl` | `scripts/syscall.tbl` |
| 计数钩子 | `arch/x86/entry/common.c` | `arch/riscv/kernel/traps.c` |
| 陷入指令 | `syscall` | `ecall` |
| 调用号 | `rax` | `a7` |

补丁：`patches/0001-riscv-syscalls.patch`（基于 linux-6.12.107）。

### 应用

```bash
cd ~/riscv/linux-6.12.107
patch -p1 < ~/rCore-Tutorial-Camp/Day5/patches/0001-riscv-syscalls.patch
```

### 配置与编译

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.83.0

./scripts/config -e SAMPLES -e SAMPLES_RUST -e SAMPLE_RUST_SYSCALLS
make ARCH=riscv LLVM=1 olddefconfig
make ARCH=riscv LLVM=1 -j20
```

### 验收

```bash
nm vmlinux | grep -E 'rust_(get_taskinfo|get_syscall_count|reset_syscall_counts|syscall_stats_inc)'
```

四个都应为 `T`。

## P2b：RISC-V 应用调用 `get_taskinfo`

`init.rs` 做三件事：

1. 调 463，拿 `tid / pid / comm`；
2. 用 64 号系统调用 `write` 打印；
3. 用 142 号 `reboot` 关机。

RISC-V 的系统调用号（asm-generic）和 x86 不同：

| 调用 | x86 | RISC-V |
|---|---|---|
| `write` | 1 | **64** |
| `exit` | 60 | **93** |
| `getpid` | 39 | **172** |
| `execve` | 59 | **221** |
| `reboot` | 169 | **142** |
| 新增 | 463–465 | 463–465 |

### 编译

```bash
cd ~/rCore-Tutorial-Camp/Day5

rustc --target riscv64gc-unknown-linux-gnu -O \
  -C panic=abort \
  -C link-arg=-nostartfiles \
  -C link-arg=-static \
  -C link-arg=-no-pie \
  -C linker=riscv64-linux-gnu-gcc \
  -o init init.rs
```

### 打包与启动

```bash
rm -rf /tmp/rdinit
mkdir -p /tmp/rdinit
cp init /tmp/rdinit/init
(cd /tmp/rdinit && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9) \
  > /tmp/rdinit.cpio.gz

~/opt/qemu-7.0.0/bin/qemu-system-riscv64 \
  -machine virt -m 1G -smp 2 -nographic \
  -kernel ~/riscv/linux-6.12.107/arch/riscv/boot/Image \
  -initrd /tmp/rdinit.cpio.gz \
  -append "console=ttyS0"
```

预期：

```
[    1.xxx] Run /init as init process
[    1.xxx] get_taskinfo ret=0
[    1.xxx] tid=1 pid=1 comm=init
[    1.xxx] reboot: Power down
```

`ret` 如果是负数补码（比如 `-38` 显示成很大的数），对应：

* `-38`：`ENOSYS`，表项没生效；
* `-22`：`EINVAL`，`len` 不对；
* `-14`：`EFAULT`，用户指针不可写。

## 下一步 P2c

`reset → app_a → app_b`，统计多个应用的系统调用次数。RISC-V 没有现成的 busybox，`/init` 要用 Rust 写，可以用 `execve(221)` 把三个程序串起来。
