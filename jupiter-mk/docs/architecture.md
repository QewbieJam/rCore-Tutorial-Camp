# jupiter-mk architecture

## 目标

面向 **Milk-V Jupiter v1.1**（SpacemiT K1/M1，RV64GCVB，Sv39）的小型微内核，
同时支持 QEMU `virt` 开发。内核只实现机制，服务在用户态。

```text
┌──────────────────────────── 用户态 ────────────────────────────┐
│  shell  ──IPC──►  console server  ──MMIO──►  UART              │
│    │                    ▲                                      │
│    └── spawn ──► hello  ──IPC──┘                               │
└───────────────────────────────┬────────────────────────────────┘
                                │ syscall / trap
┌───────────────────────────────▼────────────────────────────────┐
│  kernel: trap, tasks, scheduler, Sv39, frames, heap, IPC       │
└────────────────────────────────────────────────────────────────┘
```

## 启动路径

```text
QEMU:
  OpenSBI -> ELF at 0x8020_0000 (S-mode)

Jupiter v1.1:
  BootROM -> FSBL -> OpenSBI (M-mode) -> U-Boot (S-mode)
          -> jupiter-kernel ELF / binary at 0x2000_0000 (S-mode)
```

内核只把 SBI 当作早期启动设施：

- DBCN / legacy putchar：启动日志（进入用户态后不再使用）
- TIME：定时器
- SRST / legacy shutdown：关机

UART、shell 和应用程序都不在内核里。

## 内存管理

### 物理页帧

- 4 KiB 页帧，位图分配器，单 hart。
- 只管理 RAM 的前 2 GiB；由 `board` 常量给出范围。
- 内核镜像结束地址之前全部保留给内核、OpenSBI/U-Boot 和固件。
- 分配时清零，防止跨任务数据泄漏。
- 任务退出后，`schedule()` 在后续调度中回收僵尸任务的地址空间和内核栈。

### Sv39 地址空间

- 内核使用恒等映射：`VA == PA`。
- 内核 RAM 使用 2 MiB 大页，减少每个用户页表的页表页数量。
- 每个用户页表都包含这份 supervisor-only 恒等映射，因此：
  - trap 不需要切换 `satp`；
  - 内核栈、内核堆在任何任务上下文中都可直接访问；
  - 不需要用户态 trampoline。
- 用户映射从 `0x1_0000_0000` 开始，避开前 2 GiB 物理内存。
- 用户栈：`0x1_1000_0000` 向下 8 页。
- 用户程序是静态 RV64 ELF，由内核内嵌并通过 `mm/loader.rs` 加载。

### 页表项

仅支持 4 KiB 叶子和 2 MiB 大页；没有 1 GiB 大页、没有按需分页、没有
copy-on-write。叶子 PTE 预置 A/D 位，避免依赖硬件的 Svadu 行为；非叶子
页表指针不带 A/D。

## 任务与调度

`Task` 包含：

- pid、名字、状态
- `TaskContext`：ra/sp/s0..s11，供 `__switch` 保存内核执行流
- 内核栈 64 KiB（从内核堆分配，16 字节对齐）
- 栈顶固定存放 `TrapContext`（34 个机器字）
- `MemorySet`：用户页表及拥有的用户页帧
- capability 槽表
- IPC 暂存消息和结果

状态机：

```text
Ready -> Running -> Ready          (yield / timer)
Running -> BlockedSend(ep) -> Ready
Running -> BlockedRecv(ep) -> Ready
Running -> Zombie -> 资源回收
```

调度器是单 hart 轮转调度：

- 定时器每 10 ms 触发一次 supervisor timer 中断。
- 内核在 S-mode 关闭 `sstatus.SIE`，因此不会重入 trap。
- 中断只在用户态发生；trap 入口保存全部寄存器，调用 Rust handler。
- `schedule()` 选择下一个 Ready 任务，切换 `satp`，执行 `__switch`。
- 没有可运行任务时切回 boot/idle 上下文；若 idle 也没有任务可运行则内核 panic
  （当前不支持空闲等待）。

## Trap

`sscratch` 在用户态运行时保存当前任务内核栈顶。

```text
trap from U-mode:
  csrrw sp, sscratch, sp     # sp = kernel stack top
  allocate TrapContext
  save x1, x3..x31, user sp, sstatus, sepc
  trap_handler(cx)

trap return:
  __restore(cx)
  csrw sscratch, kstack_top
  restore sstatus/sepc/regs
  ld sp, user_sp
  sret
```

异常处理：

- `ecall` from U-mode：系统调用，`sepc += 4`，返回值写 `a0`。
- supervisor timer：重新装载定时器并抢占当前任务。
- 其他异常：打印 `sepc/stval`，终止当前任务。

## IPC

### 对象

- `Endpoint`：sender 队列 + receiver 队列。
- `Message`：`tag + 4 × usize`，固定 40 字节。
- `Cap`：目前只有 `Cap::Endpoint(id)`，每个任务 8 个槽。
- 内核启动时创建两个 endpoint：
  - cap 0：console 请求（shell/hello -> console）
  - cap 1：console 输入（console -> shell）
- `spawn` 继承父任务的能力槽。

### 语义

- `send(cap, msg)`：有等待的 receiver 时直接交付并唤醒；否则阻塞，消息暂存在
  发送者 TCB 中。
- `recv(cap, msg)`：有等待的 sender 时取走消息并唤醒 sender；否则阻塞。
- `try_recv` / `try_send`：非阻塞版本。
- 唤醒时把消息复制到接收者 TCB，系统调用返回后再按页表翻译写入用户缓冲区。

当前没有：

- 能力转移（随消息传递 cap）
- reply / call 语义
- 超时
- 多 hart 下的队列同步（当前靠单 hart + 关中断）

## 设备

console server 是唯一拥有 UART 页的任务：

- QEMU：`0x1000_0000`
- Jupiter：`0xd401_7000`（8250/XScale 兼容，115200 8N1）

内核在 `spawn` console 镜像时把该物理页以 `U|R|W` 恒等映射进 console 的
地址空间。其他任务没有这个映射。通用设备能力（MMIO cap、IRQ cap）是后续
工作。

## 系统调用 ABI

见根目录 README 的表格。用户库 `user/src/lib.rs` 与内核
`kernel/src/syscall.rs` 的编号必须保持一致。

## 后续方向

1. **Jupiter 实板验证**：U-Boot 加载、UART、定时器、内存范围。
2. **设备能力**：MMIO cap、IRQ cap、PLIC 转发，允许用户态块设备/网络驱动。
3. **IPC 增强**：cap 转移、reply 对象、批量消息/共享内存。
4. **地址空间**：按需分页、COW、用户指针范围校验、`fork`/`exec` 或 `spawn` 文件。
5. **多 hart**：每 hart 调度器、IPI、锁与 TLB shootdown。
6. **服务化**：init、块设备服务、只读文件系统、程序加载器。
