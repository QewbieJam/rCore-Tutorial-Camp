
// PTE：
// entries[0x802] = 0x20080b     （0x802 << 10 | 0b1011）
// entries[0]     = 0x201400b    （0x8050 << 10 | 0b1011）
// 其中 entries 是私有字段


// 私有字段
struct PageTable {
    entries: Vec<usize>,
}

impl PageTable {
    pub fn new(size: usize) -> Self {
        PageTable { entries: vec![0; size] }
    }

    // 实现 map, 模拟硬件构造 PTE 的过程
    pub fn map(&mut self, vpn: usize, ppn: usize, flags: usize) {
        // RISC-V PTE: 高 44 位 PPN，低 10 位 是 Flags
        let pte = (ppn << 10) | flags;
        // 将构造好的 PTE 存入对应的虚拟页号（vpn）索引处
        self.entries[vpn] = pte;
    }
}


fn main() {

    // 初始化一个大小为 0x1000（4096）的页表，确保索引 0x802 不会越界
    let mut pt = PageTable::new(0x1000);

    // 映射 1：虚拟页号 0x802 -> 物理页号 0x802, 权限 0b1011 (Valid=1, Read=1, Write=1)
    pt.map(0x802, 0x802, 0b1011);
    // 映射 2：虚拟页号 0x0 -> 物理页号 0x8050, 权限 0b1011
    pt.map(0x0, 0x8050, 0b1011);

    println!("{:#x}",pt.entries[0x802]);
    println!("{:#x}",pt.entries[0]);

}
