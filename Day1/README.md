# Day1：Rust 内核模块 —— hello / sleep 5s / goodbye

## 目标

用 Rust 写一个 Linux 可加载内核模块：

1. `insmod` 时打印 `Hello World from Rust kernel module!`
2. 睡眠 5 秒（`msleep(5000)`）
3. `rmmod` 时打印 `Goodbye from Rust kernel module!`

最后在 QEMU 中加载模块并查看串口输出。

## 环境

| 项目 | 版本 |
|---|---|
| 宿主 | Debian 13 |
| 内核源码 | linux-6.12.107 |
| 编译器 | clang / LLVM 19 |
| Rust | 1.83.0（含 `rust-src`、`llvm-tools`） |
| bindgen | 0.65.1 |

前提：内核必须启用 `CONFIG_RUST=y`，并且完整编译过一次。模块编译依赖内核树里的 `rust/` 产物（`kernel` crate 和宏库），只有 `linux-headers` 是不够的。

## 代码

`rust_hello.rs` 的关键部分：

```rust
impl kernel::Module for RustHelloWorld {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello World from Rust kernel module!\n");
        pr_info!("Now start to sleep!\n");

        // SAFETY: init 运行在进程上下文，允许睡眠。
        unsafe { kernel::bindings::msleep(5000) };

        Ok(RustHelloWorld)
    }
}

impl Drop for RustHelloWorld {
    fn drop(&mut self) {
        pr_info!("Goodbye from Rust kernel module!\n");
    }
}
```

* `impl kernel::Module` 是加载时回调，相当于 C 的 `module_init`；
* `impl Drop` 是卸载时回调，相当于 `module_exit`；
* 内核代码里不能调用 `sleep()` 这类用户态系统调用，直接调用内核函数 `msleep()`；
* 模块里没有 `fn main`。

## 编译

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.83.0

make -C "$HOME/linux-6.12.107" M="$PWD" LLVM=1 modules
```

`KDIR` 指向**完整编译过**的内核源码树，默认 `$HOME/linux-6.12.107`。

## 在 QEMU 里运行

```bash
# 1. 临时根目录
rm -rf /tmp/ird
mkdir -p /tmp/ird/{bin,lib64,lib/x86_64-linux-gnu,proc,sys,dev}

# 2. busybox 和依赖库
cp /usr/bin/busybox /tmp/ird/bin/
cp /lib64/ld-linux-x86-64.so.2 /tmp/ird/lib64/
cp /lib/x86_64-linux-gnu/libc.so.6 \
   /lib/x86_64-linux-gnu/libresolv.so.2 /tmp/ird/lib/x86_64-linux-gnu/

# 3. init 和模块
cp initramfs/init rust_hello.ko /tmp/ird/
chmod +x /tmp/ird/init

# 4. 打包
(cd /tmp/ird && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9) \
  > initramfs.cpio.gz

# 5. 启动
qemu-system-x86_64 -m 1G -smp 2 -accel tcg,thread=multi \
  -kernel "$HOME/linux-6.12.107/arch/x86/boot/bzImage" \
  -initrd "$PWD/initramfs.cpio.gz" \
  -append "console=ttyS0 panic=-1" -nographic -no-reboot
```

`initramfs/init` 会依次执行 `insmod`、`dmesg`、`rmmod`，然后关机。

## 预期输出

```
=== insmod ===
[    1.7xxxxx] Hello World from Rust kernel module!
[    1.7xxxxx] Now start to sleep!
（约 5 秒）
[    1.8xxxxx] Goodbye from Rust kernel module!
=== done ===
```

`dmesg | tail` 在 `insmod` 后就能看到前两行；goodbye 在 `rmmod` 之后出现。

## 踩坑记录

* **不能用 `cargo build`**：`kernel` crate、`module!`、`pr_info!` 只在 kbuild 里提供，cargo 工程里全部找不到。
* **Makefile 缩进必须是 Tab**，否则报 `missing separator`。
* **`obj-m` 的文件名要和 `.rs` 文件名一致**：`obj-m := rust_hello.o` 对应 `rust_hello.rs`。
* **`KDIR` 不能指向 `/lib/modules/.../build`**（发行版头文件包），必须指向自己编译过的源码树，否则报 `can't find crate for kernel`。
* **模块和内核版本必须一致**：`make -s kernelrelease` 要和 `uname -r` 对得上，否则 `insmod` 报 `invalid module format`。
* **`make mrproper` 会删掉 `.config` 和 `rust/` 等构建产物**，之后再编模块必须先重编内核。切换架构建议用独立构建目录 `O=../build-xxx`。
* **QEMU 里 `insmod` 卡 5 秒是正常的**，就是 `msleep(5000)`。
