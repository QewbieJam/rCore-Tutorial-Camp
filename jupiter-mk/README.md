# jupiter-mk

一个用 Rust 编写的 RISC-V 微内核操作系统，目标硬件是 **Milk-V Jupiter v1.1**
（SpacemiT K1/M1，8 核 RV64GCVB，Sv39），同时提供 QEMU `virt` 开发目标。

内核只保留机制：

- 启动与陷入处理
- 物理页帧分配、内核堆、Sv39 地址空间
- 任务、内核栈、轮转调度与定时抢占
- 同步 IPC（endpoint / message / capability 槽位）

驱动和服务在用户态：

- `console`：用户态 8250/XScale UART 驱动，唯一直接访问 UART 的任务
- `shell`：通过 IPC 收发字符，支持 `help`、`hello`、`pid`、`mem`
- `hello`：由 shell 通过 `spawn` 创建，证明任务创建、IPC 和退出回收

内核启动早期只通过 SBI DBCN 输出调试信息；进入用户态后，终端完全由
用户态 console 服务接管。

## 快速开始

依赖：

- Rust stable（含 `rust-src`、`llvm-tools`）
- `riscv64gc-unknown-none-elf` target
- `qemu-system-riscv64`
- 可选：`riscv64-unknown-elf-binutils`（查看 ELF）

```bash
cd ~/rCore-Tutorial-Camp/jupiter-mk
make run
```

退出 QEMU：`Ctrl-A` 然后 `X`。

示例会话：

```text
[console] uart driver online

[shell] jupiter-mk user shell
type 'help' for commands
> help
commands:
  help   show this help
  hello  spawn the hello user program
  pid    print shell pid
  mem    print free physical frames
> hello
spawned hello, pid 3
hello from user space!
[task] pid 3 (hello) exited with 0
> mem
free frames: 28067
```

## 目录结构

```text
jupiter-mk/
├── kernel/                 # 微内核
│   ├── src/
│   │   ├── entry.asm       # S-mode 入口
│   │   ├── main.rs
│   │   ├── trap/           # TrapContext、trap.S、定时器
│   │   ├── task/           # TCB、内核栈、__switch、调度、IPC 交付
│   │   ├── mm/             # 页帧、堆、Sv39 页表、ELF 加载器
│   │   ├── ipc.rs          # Endpoint / Message / Cap
│   │   ├── syscall.rs      # 系统调用分发
│   │   ├── board/          # QEMU virt 与 Jupiter 板级常量
│   │   └── user_images.rs  # 内嵌用户程序
│   ├── linker-qemu.ld
│   └── linker-jupiter.ld
├── user/                   # 用户态工作区
│   ├── src/lib.rs          # syscall 封装、Message、panic
│   └── src/bin/            # console / shell / hello
├── docs/
│   ├── architecture.md
│   └── jupiter-boot.md
└── Makefile
```

## 构建目标

```bash
make build                 # 先构建 user，再构建 QEMU 内核
make run                   # QEMU virt
make kernel BOARD=jupiter  # Jupiter 版本，链接到 0x2000_0000
make debug                 # QEMU + gdbstub
make clean
```

Jupiter 实板启动步骤见 [`docs/jupiter-boot.md`](docs/jupiter-boot.md)。

## 系统调用（当前）

| 编号 | 名称 | 参数 | 说明 |
| --- | --- | --- | --- |
| 0 | `exit` | code | 退出当前任务，资源由调度器回收 |
| 1 | `yield` | - | 让出 CPU |
| 2 | `getpid` | - | 返回 pid |
| 3 | `debug_write` | buf, len | 临时调试输出（SBI） |
| 4 | `send` | cap, msg | 阻塞发送 |
| 5 | `recv` | cap, msg | 阻塞接收 |
| 6 | `try_recv` | cap, msg | 非阻塞接收 |
| 7 | `spawn` | image_id | 创建内嵌用户程序 |
| 8 | `uart_base` | - | 返回板级 UART 物理地址 |
| 9 | `try_send` | cap, msg | 非阻塞发送 |
| 10 | `meminfo` | - | 返回空闲物理页帧数 |

`Message` 为 5 个机器字：`tag` + 4 个 `usize`，共 40 字节。字符串在
`words[0]` 中携带长度，最多 24 字节负载；更长的输出由用户库分片。

## 内存布局

```text
QEMU virt
  0x8000_0000  RAM / 内核镜像（链接到 0x8020_0000）
  0x8800_0000  128 MiB 测试 RAM 结束

Jupiter v1.1
  0x0000_0000  DRAM 起始（低 512 MiB 保留给 OpenSBI/U-Boot）
  0x2000_0000  内核镜像（U-Boot 加载地址）
  0x8000_0000  当前页帧分配器管理的 2 GiB 上限

用户空间
  0x1_0000_0000  用户程序链接地址
  0x1_1000_0000  用户栈顶（向下 8 页）
```

内核空间使用 Sv39 恒等映射（VA == PA），每个用户页表都包含这份
supervisor-only 的恒等映射，因此陷入和上下文切换不需要切换 `satp`。
内核 RAM 使用 2 MiB 大页映射，减少每个用户地址空间的页表开销。

## 已知限制

- 单 hart；尚未实现 SMP。
- 页帧分配器只管理前 2 GiB；尚未解析 DTB 的 memory/reserved-memory。
- `spawn` 使用内核内嵌镜像表，还没有文件系统或 initrd。
- 设备能力目前只对 console 任务映射 UART 页；还没有通用的 MMIO capability。
- IPC 只支持固定 40 字节消息，尚未支持能力转移和 reply 对象。
- `spawn` 继承父任务的全部能力槽，尚未支持能力裁剪/转移。
- 用户指针按页表翻译逐字拷贝，尚未做完整的范围校验/缺页处理。
- 僵尸任务在下一次调度时回收；当前任务的栈要等切换后才能释放。

## 里程碑

- **M0** 启动、SBI 控制台、panic、板级抽象 — 完成
- **M1** 位图页帧分配器、内核堆、自测 — 完成
- **M2** Sv39 页表、内核恒等映射、用户地址空间 — 完成
- **M3** trap、U 态、内核栈、轮转调度、定时抢占 — 完成
- **M4** capability 槽、endpoint、同步 IPC — 完成
- **M5** 用户态 console、shell、hello、spawn/exit/回收 — 完成
- **M6** Jupiter 实板 U-Boot 启动、UART/定时器验证 — 待实板
- **M7** 块设备服务、只读文件系统、更多应用 — 计划中
