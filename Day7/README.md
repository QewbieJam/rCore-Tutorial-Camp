# Day7 — 最小编译器（Rust 实现）

用 Rust 写一个能编译**函数调用**的小编译器：源码 → RISC-V 汇编 → 在 QEMU 上跑。

## 目录

| 文件 | 作用 |
|---|---|
| `src/main.rs` | 编译器本体：词法 → 递归下降语法 → AST → 代码生成（单文件） |
| `runtime.S` | 裸机运行时：`_start`（立栈+关机）、`putchar/puts_z/print_int/print_nl/print` |
| `linker.ld` | 链接脚本：基址 `0x80200000`、入口 `_start`、64KB 栈 |
| `build.sh` | 一条龙：`.mc → .s → .elf → qemu` |
| `hello.s` | 脚手架自检（不经过编译器，直接手写 main） |
| `examples/test.mc` | 全里程碑自测程序 |

## 用法

```bash
./build.sh hello.s              # 自检脚手架
./build.sh examples/test.mc     # 跑编译器
```

## 已验证输出

```
$ ./build.sh examples/test.mc
6      # add3(1,2,3)        函数调用
55     # fib(10)            递归
120    # fact(5)            while + 赋值
11     # x + 4 * 2          运算优先级
7      # add3(x,3,1)        变量传参
55     # 1+2+...+10         循环求和
1      # if 分支
[exit=0]
```

## 支持的语言子集

```
fn name(a, b) { ... }       函数定义
let x = expr;               局部变量
x = expr;                   赋值
return expr;                返回
if (cond) { } else { }      分支
while (cond) { }            循环
print(expr);                打印（运行时函数）
表达式: + - * /  < > <= >= == !=  ( )  整数  变量  调用
```

## 契约（编译器 ↔ 运行时）

- 入口 `_start` 在 `runtime.S`，它 `la sp, _stack_top` 后 `call main`
- 编译器生成的函数遵循 RISC-V SysV：
  - 参数 `a0-a7`，返回 `a0`，`sp` 16 字节对齐
  - **非叶子函数必须保存 `ra`**（调了别人的函数就是非叶子）
  - 用 `s0` 当帧指针，**槽位一律 `s0` 相对**（`sp` 会因压栈临时变动）
- 可直接 call：`print(x)`（整数+换行）、`print_int(x)`、`putchar(c)`、`print_nl()`

## 实测踩过的坑

1. **关机别用 SRST**（`a7=0x53525354`）——QEMU 7.0 + OpenSBI v1.0 返回 `-2 NOT_SUPPORTED`；用 legacy `a7=8`。
2. **栈帧里"保存区"和"缓冲区"别重叠**：`sd ra, 40(sp)` 的槽是 `s0-8`，缓冲区从 `s0-1` 往下写会打爆返回地址。
3. **非叶子函数不存 `ra`** → `ret` 回到最后一次 `call` 之后，表现为"打印对了然后死循环"。
4. **局部变量偏移不能用 `sp`**：表达式求值时 `sp` 会压栈变动，必须 `s0` 相对（否则 `add3(1,2,3)` 算成 4）。
5. **函数调用参数先全压栈再逆序弹进 `a0-a7`**，否则算后面的参数时前面的寄存器被嵌套调用破坏。

## 代码生成规则（读代码时对照）

- 每个表达式的结果都放 `a0`
- 二元运算：左值压栈 → 算右值 → `a1 = a0` → 弹回 `a0` → `op a0, a0, a1`
- 函数调用：所有参数依次压栈，再按 `ld a{i}, 16*(n-1-i)(sp)` 装寄存器，最后 `addi sp, sp, 16*n`
- 帧布局：`-8(s0)`=ra，`-16(s0)`=旧 s0，`-(24+8*i)(s0)`=第 i 个变量
