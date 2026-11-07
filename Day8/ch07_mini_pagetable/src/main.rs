

// ==============
// 1. 模拟物理内存
// ==============
struct PhysMem {
    base: usize,
    data: Vec<u8>,
}
impl PhysMem {
    pub fn new(base: usize, size_mb: usize) -> Self {
        let byte_size = size_mb * 1024 * 1024;
        Self {
            base,
            data: vec![0; byte_size],
        }
    }

    pub fn read_u64(&self, addr: usize) -> Option<u64> {
        if addr < self.base || addr + 8 > self.base + self.data.len() {
            return None;
        }
    }
}

struct PageTable {
    phys_mem: PhysMem,
    root_ppn: usize,    // 根页表（L2）的物理页号
}

impl PageTable {
    pub fn new(phys_mem: PhysMem, root_ppn: usize) -> Self {
        Self { phys_mem, root_ppn }
    }

    fn read_pte(&self, ppn:usize, idx: usize) -> Option<usize> {

    }

    // 【核心】Sv39 三级地址翻译
    fn translate(&self, va: usize) -> Option<usize> {
        // 1. 划分 vpn[2-0], 以及 offset
        let vpn_2 = (va >> 30) & 0x1FF;
        let vpn_1 = (va >> 21) & 0x1FF;
        let vpn_0 = (va >> 12) & 0x1FF;
        let offset = va & 0xFFF;

        // 2. 第一级页表

    }
}

fn main() {

    // 初始化 8MB 物理内存，基址 0x8020_0000
    let mut phys_mem = PhysMem::new(0x8020_0000, 8);


}
