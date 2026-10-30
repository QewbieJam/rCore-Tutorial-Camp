use std::sync::{Arc, Mutex};
use std::thread;

 // ** 主线程 & 子线程 同属于一个进程
 // ** 共享 同一个虚拟地址空间，故 Arc 指向的对内存地址是一样的
 // ** 共享 文件描述符、信号处理 等 进程级资源
 // ** 主线程 or 子线程 表现为 LWP, 在内核中表现为共享 内存空间 的 多个执行流
 //
 // *** 子线程需要自己独立的栈（stack）
 // *** 子线程需要自己独立的程序计数器（PC）和通用寄存器状态

    // {{{ 线程切换时发生 上下文切换！！！}}}
    // 1. 保存（Store）：操作系统把线程 A 当前 x1~x31 和 pc 里的值，全部用 sw / sd 指令保存到内存中（通常就保存在刚才分配的那个新栈的底部，或者专门的 Context 结构体里）。
    // 2. 恢复（Load）：操作系统把线程 B 之前保存在内存里的值，用 lw / ld 指令加载回这 32 个物理寄存器和 pc 中。
    // 3. 跳转：执行 ret 或修改 pc，CPU 开始执行线程 B 的代码。

 // *** 新线程需要接受操作系统OS 调度器Scheduler 管理




fn main() {

    // ** 主线程 执行 main

    // 负责“多对一共享”, Mutex 负责提供安全“可变性”
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));     // count = 1
    // 克隆指针，引用计数器 +1
    let data_clone = Arc::clone(&data);     // count = 2


    // ** 创建子线程 —— 关键字 thread::spawn  <-> lock()

    thread::spawn(move || { // ** 子线程从 {} 这里开始，触发了 system call
        // 先加锁
        // 修改数据
        let mut guard = data_clone.lock().unwrap();
        guard.push(4);
    }).join().unwrap();         // ** 子线程 over， count = 1

    // ** 主线程 continue
    // data_clone 被销毁
    println!("{:?}",data.lock().unwrap());

}
