
// BASIC

// 0x80554010 - 0x80200000 = 0x354010   ←   下标
// ********真实 QEMU	内核 map() → PageTableEntry::new(ppn, flags)********	0x20155c01   ←   PTE
// 0x20155c01 / 1024 = 0x80557  拆除页号，余数 1 标志位（V=1）
// 0x80557 << 12 = 0x80557000   变下一级地址


// LOG:

// 26.11.10 —— 1.0 - 仅仅可以翻译最终物理地址，无 - 检查叶子节点
// 26.11.11 —— 2.0 - 调用辅助函数检查叶子节点，并翻译最终物理地址


// TARGET:

 // 1.0 - 仅仅可以翻译最终物理地址，无 - 检查叶子节点
 // 2.0 - 检查叶子节点 + 逐级打印
 // 3.0 - 实现 map()函数 — fn map(&mut self, table_ppn: usize, idx: usize, next_ppn: usize, flags: usize)
 // 4.0 - 自动建表（不写死 0x80554 / 0x80557 / 0x80558，由程序自己分配。）




// ==============
// 1. 模拟物理内存
// ==============
struct PhysMem {
    base: usize,            // 物理内存起始地址
    data: Vec<u8>,          // 实际存储的字节数组（缓冲区）
}
impl PhysMem {
    pub fn new(base: usize, size_mb: usize) -> Self {       // 初始化函数
        let byte_size = size_mb * 1024 * 1024;        // 将 B 转换成 MB
        Self {
            base,
            data: vec![0; byte_size],
        }               // 初始化全零缓冲区，返回结构体
    }

    pub fn read_u64(&self, addr: usize) -> Option<u64> {
        //  边界检查
        if addr < self.base || addr + 8 > self.base + self.data.len() {  // 判断是否会溢出（？）
            return None;
        }

        let offset = addr - self.base; // 把 物理地址 转换成 Vec 下标
        // 取 8 字节，按小端拼成 u64
        Some(u64::from_le_bytes(self.data[offset..offset + 8].try_into().unwrap()))
        // 1. 切 8 字节
        // 2. 变成 [u8; 8]
        // 3. 拼成 u64
        // 4. 包装返回
    }

    pub fn write_u64(&mut self, addr:usize, val: u64) {
        let offset = addr - self.base;
        self.data[offset..offset + 8].copy_from_slice(&val.to_le_bytes())
    }
}

// ==============
// 2. 物理页分配器（Frame Allocator）【3.0新增】
// ==============

struct FrameAlloc {
    next: usize,
}

impl FrameAlloc {
    // 初始化分配器，指定起始物理地址
    pub fn new(start_addr: usize) -> Self {
        Self { next: start_addr }
    }

    // 分配一个物理页（4KB），返回其物理地址
    pub fn alloc_page(&mut self) -> usize {
        let addr = self.next;
        self.next += 0x1000;        // 指针后移 4KB
        addr
    }

}

// ==============
// 3. 页表管理器（Sv39 三级）
// ==============

struct PageTable {
    phys_mem: PhysMem,
    root_ppn: usize,    // 根页表（L2）的物理页号 PPN
}

impl PageTable {
    pub fn new(phys_mem: PhysMem, root_ppn: usize) -> Self {
        Self { phys_mem, root_ppn }
    }

    // 【核心】map 函数：在指定页表的指定索引处写入一个 PTE
    // table_ppn: 当前页表的物理页号 (PPN)
    // idx:       页表中的索引 (0-511)
    // next_ppn:  下一级页表或最终数据页的物理页号
    // flags:     权限标志位 (如 0b001, 0b1011 等)
    fn map(&mut self, table_ppn: usize, idx: usize, next_ppn: usize, flags: usize) {
        // 1. 计算 PTE 在物理内存中的真实地址
        let pte_addr = (table_ppn << 12) + (idx << 3);
        // 2. 构造 PTE：PPN 左移 10 位，然后与 flags 按位或
        let pte = ((next_ppn << 10) | flags) as u64;
        // 3. 写入物理内存
        self.phys_mem.write_u64(pte_addr, pte);
        // 打印调试信息
        println!("[MAP] table_ppn={:#x} idx={} -> pte_addr={:#x}, pte={:#x}", table_ppn, idx, pte_addr, pte);
    }

    // 辅助方法 read_pte()
    // 给定一个页表的 PPN 和 idx，读取其中的 PTE
    fn read_pte(&self, ppn:usize, idx: usize) -> Option<(usize, usize)> {
        // 计算该 PTE 在物理内存中的真实地址
        let pte_addr = (ppn << 12) + (idx  << 3);
        // 从物理内存读取，如果越界则返回 None
        let pte = self.phys_mem.read_u64(pte_addr)? as usize;
        // 检查 Valid 位（最低位是否为 1）
        if pte & 1 == 0 { return None; }
        Some((pte_addr, pte))
    }


    // 【核心】Sv39 三级地址翻译
    fn translate(&self, va: usize) -> Option<usize> {

        // 2.0 调用辅助函数检查叶子节点，并翻译最终物理地址

        let mut ppn = self.root_ppn;
        let levels = ["L2", "L1", "L0"];

        for(level_idx, level_name) in levels.iter().enumerate() {
            let vpn = (va >> (12 + (2 - level_idx) * 9)) & 0x1FF;

            // 调用辅助函数，解构 addr 和 pte，使用到了元组
            let (pte_addr, pte) = self.read_pte(ppn, vpn)?;

            let is_leaf = (pte & 0b1110) != 0;
            let next_ppn = pte >> 10;
            let flags = pte & 0x3FF;

            if is_leaf {
                println!("[{}] ppn={:#x} idx={} addr={:#x} pte={:#x} -> LEAF, flags={:#x}",
                        level_name, ppn, vpn, pte_addr, pte, flags);
                let offset = va & 0xFFF;
                let pa = (next_ppn << 12) | offset;
                println!("[PA] {:#x}", pa);
                return Some(pa);
            } else {
                println!("[{}] ppn={:#x} idx={} addr={:#x} pte={:#x} -> next ppn {:#x}",
                         level_name, ppn, vpn, pte_addr, pte, next_ppn);
                ppn = next_ppn;
            }
        }
        None

        // 1.0 - 仅仅可以翻译最终物理地址，无 - 检查叶子节点

        // 1. 划分 vpn[2-0], 以及 offset
        // ** Sv39 里的 VPN（每 9 位一段）
        // let vpn_2 = (va >> 30) & 0x1FF;
        // let vpn_1 = (va >> 21) & 0x1FF;
        // let vpn_0 = (va >> 12) & 0x1FF;
        // let offset = va & 0xFFF;
        // // 2. 第一级页表（L2，根页表）
        // let pte_l2 = self.read_pte(self.root_ppn, vpn_2)?;
        // let ppn_l2 = pte_l2 >> 10;
        // // 3. 第二级页表（L1）
        // let pte_l1 = self.read_pte(ppn_l2, vpn_1)?;
        // let ppn_l1 = pte_l1 >> 10;
        // // 4. 第三级页表（L0，叶子节点）
        // let pte_l0 = self.read_pte(ppn_l1, vpn_0)?;
        //
        // // 叶子节点检查：R, W, X 至少有一个为 1（即 pte & 0b1110 != 0） still in development
        // let ppn_final = pte_l0 >> 10;
        //
        // // 5. 拼接成最终物理地址
        // Some((ppn_final << 12) | offset)
    }
}


fn main() {

    // 1. 初始化 8MB 物理内存，基址 0x8020_0000（0x8020_0000 ~ 0x80A0_0000）
    let phys_mem = PhysMem::new(0x8020_0000, 8);

    // 2. 初始化物理页分配器，起始地址设为全新的 0x8060_0000
    let mut frame_alloc = FrameAlloc::new(0x8060_0000);

    // 3. 自动分配三级页表所需的物理页，并提取 PPN
    let root_addr = frame_alloc.alloc_page();
    let l1_addr = frame_alloc.alloc_page();
    let l0_addr = frame_alloc.alloc_page();

    let root_ppn = root_addr >> 12;     // 0x80600
    let l1_ppn = l1_addr >> 12;         // 0x80601
    let l0_ppn = l0_addr >> 12;         // 0x80602

    println!("--- 自动分配页表物理页 ---");
    println!("Root PPN: {:#x} (Addr: {:#x})", root_ppn, root_addr);
    println!("L1   PPN: {:#x} (Addr: {:#x})", l1_ppn, l1_addr);
    println!("L0   PPN: {:#x} (Addr: {:#x})", l0_ppn, l0_addr);
    println!("--------------------------------\n");

    // 4. 初始化页表管理器
    let mut pt = PageTable::new(phys_mem, root_ppn);

    // 5. 自动建表流程 (完全动态，无硬编码) ———— 目前还没看懂
    println!("--- 开始自动建表 ---");
    // ④ map(root, 2, l1_ppn, 0b001) -> Valid=1, 非叶子
    pt.map(root_ppn, 2, l1_ppn, 0b001);
    // ⑤ map(l1, 1, l0_ppn, 0b001)   -> Valid=1, 非叶子
    pt.map(l1_ppn, 1, l0_ppn, 0b001);
    // ⑥ map(l0, 0, 0x80200, 0b1011) -> Valid=1, R=1, W=1 (叶子节点，指向内核 .text)
    pt.map(l0_ppn, 0, 0x80200, 0b1011);
    println!("--------------------------------\n");

    // 3.0 构造 map(&mut self, table_ppn: usize, idx: usize, next_ppn: usize, flags: usize) 函数
    // pt.map(0x80554, 2, 0x80557, 0b001);
    // pt.map(0x80557, 1, 0x80558, 0b001);
    // pt.map(0x80558, 0, 0x80200, 0b1011);

    // // 模拟"内核建表"
    // pt.phys_mem.write_u64((0x80554 << 12) + (2 << 3), 0x20155c01);
    // pt.phys_mem.write_u64((0x80557 << 12) + (1 << 3), 0x20156001);
    // pt.phys_mem.write_u64((0x80558 << 12) + (0 << 3), 0x2008000b);


    let va = 0x80200000;

    match pt.translate(va) {
        Some(pa) => {
            println!("[VA]: {:x?}", va);
            println!("[PA]: {:x?}", pa);
        }
        None => {
            panic!("翻译失败，返回 None");
        }
    }

}
