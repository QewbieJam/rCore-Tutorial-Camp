# Day7 — 从汇编到 Rust：一条能跑通的最小链路

## 这一节做了什么

1. **手写汇编**，在 Ripes/Venus 单步验证"为什么二元运算必须压栈"
2. **用 rustc 把 `.rs` 编译成 RISC-V 汇编**（`--emit asm`）
3. **对比优化前后**：`li a0, 6` vs 真实函数调用（常量折叠/内联）
4. **打通全链路**：`.rs → .s → .o → .elf → QEMU`，真机打印出 `6`

## 用到的工具

| 工具 | 版本 | 用途 |
|---|---|---|
| `rustc` | 1.99.1（target `riscv64gc-unknown-none-elf`） | `.rs` → `.s` |
| `riscv64-unknown-elf-gcc` | 8.3 | 汇编 + 链接 |
| `qemu-system-riscv64` | 7.0.0（内置 OpenSBI 1.0） | 运行 ELF |
| Ripes / Venus | — | 单步，看寄存器/内存 |

## 目录

| 路径 | 内容 |
|---|---|
| `asm/demo.s` | 手写：`42 + 7`，观察 `a0` 被覆盖、栈备份救回 |
| `asm/demo2.s` | 手写：`(2 + 3) * 4`，嵌套二元运算 |
| `rust/add3.rs` | `no_std` Rust 源码（`extern "C" { fn print_int(i64); }`） |
| `rust/add3_O0.s` | rustc `-O0` 产物，去掉伪指令，**Ripes 可直接加载** |
| `rust/add3_O3.s` | rustc `-O` 产物：`li a0, 6`（常量折叠） |
| `rust/f.c` / `f.s` | GCC 对照，看 prologue：`addi s0, sp, 32` |
| `chain/` | 全链路：`build.sh` 一条命令跑完 |

## 复现

```bash
# 1. 纯汇编实验（Ripes：File → Load Program）
#    asm/demo.s  → 看 a0 被覆盖、栈里 0x2a
#    asm/demo2.s → 看嵌套时栈的复用

# 2. Rust 编译到汇编
cd rust
rustc --target riscv64gc-unknown-none-elf --crate-type lib \
      --emit asm -O -o add3_O3.s add3.rs

# 3. 全链路运行
cd ../chain
./build.sh
# → [1/4] rustc ... [4/4] qemu
# → 6
```

## 关键结论

| 问题 | 答案 |
|---|---|
| 一条指令多长 | 4 字节（无 C 压缩扩展）；有 C 时小指令 2 字节 |
| 第 n 条指令地址 | 目标文件里 `4×n`；链接后 `段基址 + 偏移` |
| `jal ra, X` 做了什么 | `ra = PC + 4`，然后跳转 |
| `ret` 做了什么 | 只按 `ra` 跳转，**不恢复任何东西** |
| 谁恢复 ra/sp | 后面的 `ld ra, ...` 和 `addi sp, sp, ...` |
| 建帧/拆帧的名字 | prologue / epilogue（frame allocation / deallocation） |
| 代码在哪 | `.text` 段（本项目链接后 `0x80200000`） |
| 栈在哪 | `.bss` 末尾预留 64KB，`sp` 从 `_stack_top` 向下长 |

## 踩过的坑

1. **`.attribute` ISA 字符串**：rustc 1.99 生成的 `rv64i2p1_..._z...`，gcc 8.3 汇编器不认 → 删掉该行
2. **Ripes 不认伪指令**：`.section` / `.globl` / `.type` / `.size` / `.p2align` / `.file` / `.ident` 全部要删，只留标签和指令
3. **Ripes 没有 `print_int`**：`tail print_int` 要换成 `j .` 或 `ecall`
4. **Ripes 从 `0x0` 开始执行**：文件开头要补 `jal ra, main`，否则从 `add3` 跑起
5. **少了 `li sp, 0x8000`**：寄存器全 0 初始化时 `sp=0`，`addi sp,sp,-16` 会写到越界地址
6. **`tail print_int` 不带换行**：输出 `6` 会粘在提示符上，看起来像没输出
