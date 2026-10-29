# Ripes 汇编实验

在 Ripes（或 Venus）里单步观察"为什么二元运算必须压栈"。

## 文件

| 文件 | 说明 |
|---|---|
| `demo.s` | 9 条指令的实验程序（手写汇编，模拟 `let x = 42 + 7`） |
| `f.c` | 用来对照的 C 代码 |
| `f.s` | GCC 生成的汇编（对照 prologue：`addi s0, sp, 32`） |

## 怎么跑

### Ripes

1. 打开 Ripes，选 **RV64 + Single Cycle**
2. ISA Exts 勾 **M**（有乘除才需要；本 demo 只用到 add，可不勾）
3. Register Initialization 选 **All zeroes**
4. `File → Load Program…` 选 `demo.s`
5. 切 **Processor** 标签，点 **Step** 单步
6. 同时看 **Memory** 标签，跳到 `0x7ff0`

### Venus（网页版）

https://venus.cs61c.org → 粘贴 `demo.s` → Simulator → Step

## 程序与预期状态

| Step | 指令 | 执行后 sp | a0 |
|---|---|---|---|
| 1 | `li sp, 0x8000` | 0x8000 | 0 |
| 2 | `li a0, 42` | 0x8000 | 42 |
| 3 | `addi sp, sp, -16` | 0x7ff0 | 42 |
| 4 | `sd a0, 0(sp)` | 0x7ff0 | 42（内存 0x7ff0 出现 `0x2a`） |
| 5 | `li a0, 7` | 0x7ff0 | **7** ← 42 被覆盖 |
| 6 | `mv a1, a0` | 0x7ff0 | 7（a1=7） |
| 7 | `ld a0, 0(sp)` | 0x7ff0 | **42** ← 救回来 |
| 8 | `addi sp, sp, 16` | 0x8000 | 42 |
| 9 | `add a0, a0, a1` | 0x8000 | **49** |

## 关键实验：删掉压栈

把第 3、4 条（`addi sp, sp, -16` 和 `sd a0, 0(sp)`）删掉，重新 Assemble 再跑：

- 第 5 步 `li a0, 7` 之后，42 **永久丢失**
- 第 7 步 `ld a0, 0(sp)` 读到的不是 42，而是 `0x8000` 处的垃圾值（通常是 0）
- 最终 `a0 = 0 + 7 = 7`，**不是 49**

这就是"左压栈"存在的理由：`a0` 是 caller-saved，算右边时会被覆盖，左边唯一的落脚点是栈。

## 对照 GCC

```bash
riscv64-unknown-elf-gcc -march=rv64imac -mabi=lp64 -O0 -S f.c -o f.s
```

GCC 生成的叶子函数 `add3`：

```asm
add3:
	addi	sp, sp, -32
	sd	s0, 24(sp)
	addi	s0, sp, 32        # ← 和我们编译器一样：s0 = 开帧前的 sp
	...
```

注意 GCC **没有保存 `ra`**——因为 `add3` 是叶子函数（内部不调用别人）。我们的编译器统一保存，稳妥但多一条指令。
