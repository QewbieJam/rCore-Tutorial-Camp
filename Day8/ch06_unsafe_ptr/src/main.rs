


fn main() {

    // 64位 系统中， 一个 usize 大小为8字节
    let mut value: usize = 0xdeadbeaf;          // 定义一个 usize 值


    // &value 获取引用 -> as *const usize 转为不可变指针 -> as usize 转为整数
    let addr = &mut value as *mut usize as usize;     // 获取 value 地址
    //let addr = &value as *const usize as usize;



    println!("addr: {:#x}, value: {:#x}", addr, value);


    let wildcard = unsafe { *(addr as *const usize) };
    println!("wildcard: {}", wildcard);


    unsafe {
        *(addr as *mut usize) = 0xcafebabe;
    }

    let read_val_again = unsafe { *(addr as *mut usize) };

    println!("read_val_again: {}", read_val_again);

    println!("value: {}", value);



    // 模拟一个包含 4 个页表项（PTE）的连续内存块
    let page_table: [usize; 4] = [0x8020_0001, 0, 0x8021_0001, 0];

    // 获取基址 base
    let base = &page_table[0] as *const usize as usize; // 首地址

    println!("base: {:#x}, value: {:#x}", base, page_table[0]);

    // 获取页表 PTE
    let pte1 = unsafe{ *((base + 0 * 8) as *const usize) };
    let pte2 = unsafe{ *((base + 2 * 8) as *const usize) };

    println!("pte1: {:#x}", pte1);
    println!("pte2: {:#x}", pte2);

}
