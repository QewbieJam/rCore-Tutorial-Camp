
// ==============
// 1. 模拟物理内存
// ==============


// 0x80554010 - 0x80200000 = 0x354010   ←   下标

// ********真实 QEMU	内核 map() → PageTableEntry::new(ppn, flags)********	0x20155c01   ←   PTE

// 0x20155c01 / 1024 = 0x80557  拆除页号，余数 1 标志位（V=1）
// 0x80557 << 12 = 0x80557000   变下一级地址



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
// 2. 页表管理器（Sv39 三级）
// ==============

struct PageTable {
    phys_mem: PhysMem,
    root_ppn: usize,    // 根页表（L2）的物理页号 PPN
}

impl PageTable {
    pub fn new(phys_mem: PhysMem, root_ppn: usize) -> Self {
        Self { phys_mem, root_ppn }
    }

    // 辅助方法 read_pte()
    // 给定一个页表的 PPN 和 idx，读取其中的 PTE
    fn read_pte(&self, ppn:usize, idx: usize) -> Option<usize> {
        // 计算该 PTE 在物理内存中的真实地址
         let pte_addr = (ppn << 12) + (idx  << 3);
        // 从物理内存读取，如果越界则返回 None
        let pte = self.phys_mem.read_u64(pte_addr)? as usize;
        // 检查 Valid 位（最低位是否为 1）
        if pte & 1 == 0 {
            return None;
        }

        Some(pte)
    }


    // 【核心】Sv39 三级地址翻译
    fn translate(&self, va: usize) -> Option<usize> {
        // 1. 划分 vpn[2-0], 以及 offset
        // ** Sv39 里的 VPN（每 9 位一段）
        let vpn_2 = (va >> 30) & 0x1FF;
        let vpn_1 = (va >> 21) & 0x1FF;
        let vpn_0 = (va >> 12) & 0x1FF;
        let offset = va & 0xFFF;
        // 2. 第一级页表（L2，根页表）
        let pte_l2 = self.read_pte(self.root_ppn, vpn_2)?;
        let ppn_l2 = pte_l2 >> 10;
        // 3. 第二级页表（L1）
        let pte_l1 = self.read_pte(ppn_l2, vpn_1)?;
        let ppn_l1 = pte_l1 >> 10;
        // 4. 第三级页表（L0，叶子节点）
        let pte_l0 = self.read_pte(ppn_l1, vpn_0)?;

        // 叶子节点检查：R, W, X 至少有一个为 1（即 pte & 0b1110 != 0） still in development
        let ppn_final = pte_l0 >> 10;

        // 5. 拼接成最终物理地址
        Some((ppn_final << 12) | offset)
    }
}


fn main() {

    // 初始化 8MB 物理内存，基址 0x8020_0000
    let phys_mem = PhysMem::new(0x8020_0000, 8);

    let mut pt = PageTable::new(phys_mem, 0x80554);

    // 模拟"内核建表"
    pt.phys_mem.write_u64((0x80554 << 12) + (2 << 3), 0x20155c01);
    pt.phys_mem.write_u64((0x80557 << 12) + (1 << 3), 0x20156001);
    pt.phys_mem.write_u64((0x80558 << 12) + (0 << 3), 0x2008000b);


    let va = 0x80200000;

    match pt.translate(va) {
        Some(pa) => {
            println!("VA: {:x?}", va);
            println!("PA: {:x?}", pa);
        }
        None => {
            println!("--- 页表项无效或越界 ---");
            panic!("--- Sv39 页表翻译未通过 ---")
        }
    }

}
