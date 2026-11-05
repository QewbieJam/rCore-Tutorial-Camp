

// Need_TO_Know:
// PTE，64位（8字节），由 44位 的 PPN（物理页号）、10位 标志位 和 10位 保留位（全零）组成

// 其中，10位 标志位 包含了：
// 位	名称	作用
// 0	V (Valid)	1 表示这个页表项有效
// 1	R (Read)	1 表示可读
// 2	W (Write)	1 表示可写
// 3	X (Execute)	1 表示可执行
// 4	U (User)	1 表示用户态可访问
// 5	G (Global)	1 表示全局映射
// 6	A (Accessed)	1 表示被访问过
// 7	D (Dirty)	1 表示被修改过
// 8-9	RSW	留给操作系统自定义使用


// 💡 总结一下流程
//
// OS 建立页表，算出根页表的物理地址，把它的 PPN 写入 satp 寄存器。
//
// APP 在用户态运行，给出虚拟地址（VA）。
//
// CPU 硬件（MMU） 拿着当前的 satp 和 VA，自动走页表，得到物理地址（PA）。
//
// 如果发生进程切换，OS 负责把另一个进程的根页表 PPN 重新写入 satp。


// satp（Supervisor Address Translation and Protection）是一个特权寄存器，
// 只有运行在内核态（如 RISC-V 的 S 模式或 M 模式）的 OS 才能操作它。



// 1. 查找 PTE (Page Table Entry)
fn lookup(table: &[usize], idx: usize) -> Option<usize> {
    // 情况 1：越界检查（idx 超出了页表的长度）
    if idx >= table.len() {
        return None;
    }

    let pte = table[idx];

    // 情况 2：有效位检查（最低位为 0，表示该页表项无效）
    if pte & 1 == 0 {
        return None;
    }

    // 情况 3：有效，返回完整的 PTE
    Some(pte)
}

// 2. 地址转换（Translate Virtual to Physical）
fn translate(table: &[usize], idx: usize) -> Option<usize> {
    // 【核心】使用 ？ 操作符：
    // 如果 lookup 返回 None，translate 也会立即返回 None（短路求值）
    // 如果 lookup 返回 Some，则把 pte 解包赋值给变量
    let pte = lookup(table, idx)?;
    Some((pte >> 10) << 12)
}

fn print_result(idx: usize, res: Option<usize>) {
    match res {
        Some(val) => println!("idx {} -> Some({:#x})", idx, val),
        None => println!("idx {} -> None", idx),
    }
}

fn main() {
    // 构造一个模拟页表（Vec 在底层就是 &[usize] 的切片）
    // 此处精确构造 PTE 的值，以满足输出要求
    // 目标： idx 0 转换后为 0x80200000
    // 反推： 0x80200000 = (pte >> 10) << 12 => pte >> 10 = 0x80200 => pte = 0x20080000 | 1 (Valid bit) = 0x20080000
    let page_table = vec![
        0x20080001,
        0x12345670,
        0x00000000,
        0x00000000,
        0x00000000,

    ];

    // 测试三种情况
    print_result(0, translate(&page_table, 0)); // 有效项
    print_result(1, translate(&page_table, 1)); // 无效项（Valid bit = 0）
    print_result(9, translate(&page_table, 9)); // 越界（页表长度仅为 5）
}



// pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {}

