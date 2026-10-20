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

## 明天

裸机应用 B（Rust，`no_std` + `no_main` + `syscall`）还没放进来，下一步做：

* `reset`：调 465 清零；
* `app_a`：调 463 打印自己的 tid/pid/comm；
* `app_b`：调 463，再逐个调 464 打印计数；
* `init`：`reset → app_a → app_b`。

## 备注

* 本补丁基于 x86_64 的 6.12.107，其他架构需要改对应 `syscall.tbl`；
* 计数在热路径上多一次函数调用 + 原子加，演示够用，生产环境应改 per-CPU；
* Rust 工具链要求与内核匹配（本机用 1.83.0），并安装 `rust-src`、`llvm-tools`。
