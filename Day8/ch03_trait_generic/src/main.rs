// trait：定义一组共享行为/接口，impl Trait for Type 实现；
// 泛型：fn f<T>(x: T)、struct S<T>，编译期单态化，零成本抽象；
// 约束：T: Trait、where 子句；
// 静态分发 vs 动态分发：impl Trait / 泛型是静态分发，dyn Trait 是动态分发（虚表）；
// 关联类型、默认方法、trait 对象。

// 项目简介：

// 一个模拟的 “栈分配器（Bump Allocator）”， 主要演示 Trait 和 Struct 配合，以及 底层内存地址 的分配逻辑



// 1. 声明 Trait - 规定了分配器必须实现 “分配” 和 “查余量”
trait Allocator {
    fn alloc(&mut self) -> usize;          // “分配功能”
    fn remaining(&self) -> usize;          // “查余量”
}

// 2. 定义 Struct（具体数据结构）
struct StackAlloc {                        // 解释 “next” 和 “end” 的含义
    next: usize,
    end: usize,
}

// 3. 为 Struct 实现 Trait（具体逻辑实现）
impl Allocator for StackAlloc {
    fn alloc(&mut self) -> usize {
        let addr = self.next;       // 记录 当前的地址
        self.next += 0x1000;              // 指针向后移动 4KB（0x1000），通常是一页大小
        addr                              // 返回 当前的地址
    }
    fn remaining(&self) -> usize {        // 算出剩余空间
        // 计算剩余空间
        self.end - self.next
    }
}


// 第二个实现：FakeAlloc
struct FakeAlloc {
    n: usize,
}

impl Allocator for FakeAlloc {
    fn alloc(&mut self) -> usize {
        self.n += 1;
        self.n * 0x1000
    }

    fn remaining(&self) -> usize {
        0
    }
}

fn main() {
    // 测试 StackAlloc
    // 样例初始化
    let mut s = StackAlloc {            // 初始化，调用了两次 alloc, 1次 remaining
        next: 0x8020_0000,
        end: 0x8023_0000
    };

    // 调用方法
    println!("第一次分配：{:#x}", s.alloc());
    println!("第二次分配：{:#x}", s.alloc());

    // 打印剩余空间
    println!("剩余空间: {:#x}", s.remaining());

    // 测试 FakeAlloc
    let mut f = FakeAlloc { n: 0 };

    println!("{:?}", f.alloc());
    println!("{:?}", f.alloc());
    println!("剩余：{:#x}", f.remaining());

}

