# Day3：给内核加系统调用（Rust 实现）

本目录只包含**相对 `linux-6.12.107` 的改动**，不包含内核源码本身。

## 新增的系统调用

| 编号 | 名字 | 作用 |
|---|---|---|
| 463 | `get_taskinfo` | 把当前 task 的 tid / pid / comm 写到用户指针 |
| 464 | `get_syscall_count` | 返回某个系统调用被调用过的次数 |
| 465 | `reset_syscall_counts` | 清零所有系统调用计数 |

`struct taskinfo` 布局（UAPI）：

```c
struct taskinfo {
	__u32 tid;      /* 线程 id */
	__u32 pid;      /* 进程 id（tgid） */
	char comm[16];  /* task name */
};
```

## 改动清单

| 文件 | 改动 |
|---|---|
| `arch/x86/entry/syscalls/syscall_64.tbl` | 加 463/464/465 三条表项 |
| `arch/x86/entry/common.c` | 在 `x64_sys_call()` 前调用 `rust_syscall_stats_inc(unr)` |
| `include/linux/syscall_stats.h` | 声明 Rust 计数函数（新增） |
| `include/linux/syscalls.h` | 三个 `sys_*` 原型 |
| `include/uapi/linux/taskinfo.h` | `struct taskinfo`（新增） |
| `kernel/syscall_stats.c` | 三个 `SYSCALL_DEFINE` 转发壳（新增） |
| `kernel/Makefile` | `obj-y += syscall_stats.o` |
| `samples/rust/rust_syscalls.rs` | Rust 实现（新增） |
| `samples/rust/Kconfig` | 新增 `SAMPLE_RUST_SYSCALLS` |
| `samples/rust/Makefile` | 编译 `rust_syscalls.o` |

补丁：`patches/0001-add-rust-syscalls.patch`

## 设计说明

* **C 只做 ABI 胶水**：`SYSCALL_DEFINE` 负责从 `pt_regs` 取参数，函数体只调用 Rust；
* **Rust 负责全部逻辑**：`samples/rust/rust_syscalls.rs` 用 `#[no_mangle] extern "C"` 暴露 4 个符号：
  `rust_get_taskinfo`、`rust_get_syscall_count`、`rust_reset_syscall_counts`、`rust_syscall_stats_inc`；
* **计数**：入口处在真正分发前给 `nr` 加一，所以 `get_syscall_count(464)` 会把自己这次也算进去；
* 计数器是全局 `AtomicU64[512]`，跨进程累计；`reset` 清零全部；
* `get_taskinfo` 用 `kernel::current!()` 取 tid/pid，用 `bindings::__get_task_comm` 取 comm，用 `UserSlice` 写回用户态；
* 失败返回负 errno：`-EINVAL`（参数不对）、`-EFAULT`（用户拷贝失败）。

## 应用补丁

在 **linux-6.12.107 源码树根目录**执行：

```bash
cd ~/linux-6.12.107
patch -p1 < ~/rCore-Tutorial-Camp/Day3/patches/0001-add-rust-syscalls.patch
```

## 配置

**注意顺序**：必须先开 `CONFIG_SAMPLES`，否则 `samples/rust/Kconfig` 不会被解析。

```bash
./scripts/config -e SAMPLES -e SAMPLES_RUST -e SAMPLE_RUST_SYSCALLS
make LLVM=1 olddefconfig
grep -E 'CONFIG_(SAMPLES|SAMPLES_RUST|SAMPLE_RUST_SYSCALLS)=' .config
```

三个都应为 `=y`。也可以直接 `cat` 本目录的 `config.fragment` 追加到 `.config`，再 `make olddefconfig`。

## 编译

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.83.0
make LLVM=1 -j"$(nproc)"
```

## 验证

```bash
nm vmlinux | grep -E 'rust_(get_taskinfo|get_syscall_count|reset_syscall_counts|syscall_stats_inc)'
```

四个都应为 `T`，说明 Rust 对象已经链接进 vmlinux。

## 裸机应用（Rust）

`Day3/app/` 下是三个 `no_std` / `no_main` 的 Rust 程序：

| 文件 | 作用 |
|---|---|
| `mini.rs` | 内联 `syscall`、打印、`_start`、panic handler、`mem*` |
| `reset.rs` | 调 465 清零计数 |
| `app_a.rs` | 调 463 打印自己的 tid/pid/comm |
| `app_b.rs` | 调 463，再调 464 打印各系统调用计数 |

编译：

```bash
rustc --edition 2021 -O -C panic=abort -C relocation-model=static \
  -C link-arg=-nostartfiles -C link-arg=-static -C link-arg=-no-pie \
  -o app/reset app/reset.rs

rustc --edition 2021 -O -C panic=abort -C relocation-model=static \
  -C link-arg=-nostartfiles -C link-arg=-static -C link-arg=-no-pie \
  -o app/app_a app/app_a.rs

rustc --edition 2021 -O -C panic=abort -C relocation-model=static \
  -C link-arg=-nostartfiles -C link-arg=-static -C link-arg=-no-pie \
  -o app/app_b app/app_b.rs
```

生成 `app/reset`、`app/app_a`、`app/app_b`，都是静态 `ET_EXEC`。

打包 initramfs：

```bash
rm -rf /tmp/ird
mkdir -p /tmp/ird/{bin,proc,sys,dev}

# 静态 busybox：只拷这一个文件
cp /path/to/static/busybox /tmp/ird/bin/busybox

cp initramfs/init /tmp/ird/init
cp app/reset app/app_a app/app_b /tmp/ird/
chmod +x /tmp/ird/init /tmp/ird/bin/busybox \
         /tmp/ird/reset /tmp/ird/app_a /tmp/ird/app_b

(cd /tmp/ird && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9) \
  > initramfs.cpio.gz
```

如果你用的是动态链接的 busybox，还要把 `ldd` 列出的库按原路径拷进 `/tmp/ird`。

启动：

```bash
qemu-system-x86_64 -m 1G -smp 2 -accel tcg,thread=multi \
  -kernel ~/linux-6.12.107/arch/x86/boot/bzImage \
  -initrd "$PWD/initramfs.cpio.gz" \
  -append "console=ttyS0 panic=-1" -nographic -no-reboot
```

预期输出节选：

```
=== reset ===
=== A ===
A: hello
A: tid = 66
A: pid = 66
A: comm = app_a
A: Status: Finished
=== B ===
B: ret = 0
tid =68
pid =68
comm =app_b
--- syscall counts ---
write(1) = 27
...
```

`write(1) = 27` 是 reset 之后**所有进程**的 write 次数：两个 `echo`（各 2 次）、
`app_a` 的 11 次、`app_b` 读取前的 12 次，合计 27。

### 踩坑

* `print_u` 里如果写成 `&buf[i..1]`，非零数字会越界 panic，进程被 `panic_handler`
  静默 `exit(127)`；正确写法是 `&buf[i..]`；
* `-nostartfiles` 之后必须自己提供 `_start`，否则 ELF 没有入口；
* `no_std` 下没有 `memcpy/memset`，要自己实现，否则链接报 undefined；
* 打包必须在 `initramfs/` 目录里执行 `find .`，否则归档路径会多一层。

## 备注

* 本补丁基于 x86_64 的 6.12.107，其他架构需要改对应 `syscall.tbl`；
* 计数在热路径上多一次函数调用 + 原子加，演示够用，生产环境应改 per-CPU；
* Rust 工具链要求与内核匹配（本机用 1.83.0），并安装 `rust-src`、`llvm-tools`。
