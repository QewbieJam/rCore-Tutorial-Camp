# Chapter 1 学习词条（2026-10-25）

> 只保留“学了什么”的词条索引，不展开正文。
> 用于复习、检索和上传，完整内容仍在同目录各篇笔记中。

## 1. 今日主题一句话

从 **x86 转向 RISC-V** 的过程中，补齐“程序如何从源码走到硬件”的纵向链路：

```
源码 → 编译器/汇编器/链接器 → ELF → 加载 → 栈/调用约定 → 系统调用 → SBI/固件 → 硬件
```

## 2. 篇目词条

| # | 篇目 | 关键词 | 一句话 |
|---|---|---|---|
| 0 | 裸机程序与 SBI：五项能力 | ROM、固件、内核、init、入口共识 | 每一棒交接都靠“地址 + 寄存器状态 + ABI”的共识；五项能力是裸机开发的底线 |
| 1 | 为什么编译原理、OS、组成原理是核心 | 纵向系统栈、抽象层、性能/安全 | 三门课分别回答“硬件能做什么、资源怎么分、语言怎么翻译” |
| 2 | Compiler / Assembler / Linker | 编译、汇编、链接、重定位 | 编译器翻译语言，汇编器编码指令，链接器拼装并定址 |
| 3 | 栈到底是什么 | 栈帧、寄存器优先、spill | 栈是调用帧仓库，不是参数/返回值的默认位置；参数优先走寄存器 |
| 4 | QEMU `-initrd` 与 `-append` | initramfs、bootargs、console | `-initrd` 给根文件系统，`-append` 给内核命令行 |
| 5 | 内核 Image 与 init ELF 的加载位置 | 0x80200000、高地址、0x10000 | Image 按物理地址加载；initrd 临时放高地址；init ELF 在 execve 时映射到用户虚拟地址 |
| 6 | `call` 的 `auipc`/`jalr` 与 x1/x6 | ra、t1、目标地址 | 返回地址在 x1；x6 只是临时放目标地址 |
| 7 | RISC-V 函数调用规范 | a0–a7、fa0–fa7、s0–s11、16 字节对齐 | 参数用寄存器，返回值在 a0/fa0，ra 存返回地址，callee 保存 s0–s11 |
| 8 | 栈帧与调用栈 | 帧、嵌套、向下增长、线程栈 | 栈由帧组成；同一线程的帧在同一个向下增长的栈里；不同线程/内核各有栈 |
| 9 | 内存相关缺陷 | 溢出、UAF、double free、竞争 | 读/写/分配/释放/共享做错；Rust 安全代码能挡大部分，unsafe 仍要守规矩 |
| 10 | RustSBI：服务与使用 | M-mode、SBI、扩展、`-bios` | RustSBI 是 M-mode 固件；通过 SBI 给内核提供 TIME/IPI/HSM/SRST 等服务 |
| 11 | RISC-V H 扩展与 SBI 2.0/3.0 | HS/VS/VU、NACL、SSE/FWFT | H 扩展把 S 拆成 HS + VS/VU；RustSBI 覆盖 SBI 2.0 核心扩展，并提供 3.0 特型 |

## 3. 术语速查

### 启动与固件

| 词条 | 一句话 |
|---|---|
| ROM / 复位向量 | 上电后 CPU 的第一条指令地址 |
| 固件（OpenSBI / RustSBI） | M-mode 代码，提供 SBI 服务 |
| bootloader | 负责把内核从磁盘/Flash 搬到内存并跳转 |
| DTB | 设备树，告诉内核内存、串口、initrd 在哪 |
| SBI | S-mode 请求 M-mode 帮忙的接口 |
| HSM | Hart State Management，多核启停 |
| SRST | System Reset，关机/重启 |
| `-bios` | QEMU 指定 M-mode 固件 |
| `-kernel` | QEMU 指定内核镜像 |
| `-initrd` | QEMU 指定 initramfs 归档 |
| `-append` | 内核命令行（bootargs） |

### ELF 与工具

| 词条 | 一句话 |
|---|---|
| ELF | 可执行/目标文件格式 |
| section | 链接/调试视角的内容分类（.text/.data/.rodata） |
| segment | 加载视角的映射单元（PT_LOAD） |
| VMA / LMA | 虚拟地址 / 加载地址 |
| ET_REL / ET_EXEC / ET_DYN | 可重定位 / 固定可执行 / 位置无关 |
| compiler | 高级语言 → 汇编/IR |
| assembler | 汇编 → 机器码 `.o` |
| linker | 符号解析、重定位、拼装 |
| objcopy | 格式转换/抽取，`-O binary` 生成裸二进制 |
| readelf | 看 ELF 头、节、段、符号 |
| nm | 列符号表（值 + 类型 + 名字） |
| objdump | 反汇编、看节内容 |

### RISC-V 与调用约定

| 词条 | 一句话 |
|---|---|
| x0 / zero | 恒为 0 |
| x1 / ra | 返回地址 |
| x2 / sp | 栈指针，16 字节对齐 |
| x8 / s0/fp | 帧指针（可选） |
| a0–a7 | 参数 / 返回值 |
| s0–s11 | callee-saved |
| t0–t6 | caller-saved 临时寄存器 |
| `call` | `auipc ra` + `jalr ra` |
| `tail` | 不保存返回地址，目标常放 t1 |
| `ret` | `jalr zero, 0(ra)` |
| `ecall` | 陷入更高级特权级 |
| 叶子函数 | 不调用别人，可以不建栈帧 |
| 栈帧 | 一次调用在栈上占的区域 |
| guard page | 栈底保护页，越界触发缺页 |

### 特权级与虚拟化

| 词条 | 一句话 |
|---|---|
| M-mode | 固件，最高权限 |
| S-mode / HS-mode | 内核 / 带 H 扩展的内核 |
| U-mode | 用户程序 |
| VS / VU | Guest OS 内核 / Guest 用户 |
| H 扩展 | 把 S 拆成 HS，并派生 VS/VU |
| 两阶段翻译 | VS-stage（`vsatp`）+ G-stage（`hgatp`） |

### 内存缺陷

| 词条 | 一句话 |
|---|---|
| 缓冲区溢出 | 写超出边界 |
| UAF | 释放后继续用 |
| double free | 同一块释放两次 |
| 悬空指针 | 指向已失效内存 |
| 内存泄漏 | 分配后不释放 |
| 数据竞争 | 无同步的并发读写 |
| ASan / KASAN | 内存错误检测工具 |
| Miri | Rust UB 检测 |

## 4. 自检问题

1. `Image`、`rdinit.cpio.gz`、`init` 分别在哪一层？
2. `-initrd` 和 `-append` 各解决什么问题？
3. `call` 和 `tail` 的返回地址分别在哪？
4. RISC-V 的前 8 个整数参数放哪？第 9 个呢？
5. 为什么叶子函数可以不建栈帧？
6. 同一进程的两个线程共用栈吗？
7. 栈向下增长，`sp` 指向高地址还是低地址？
8. `nm` 能查依赖吗？查依赖用什么？
9. SBI 调用里 `a7`/`a6` 分别是什么？
10. H 扩展下 Guest 的 SBI 调用先到哪一层？

---

> 上传建议：只上传这份“词条”，完整笔记留在本地 vault。
