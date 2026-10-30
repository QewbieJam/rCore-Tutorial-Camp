use std::sync::{Mutex, OnceLock};
use std::thread;

// 1. 声明 “全局锁”
// OnceLock 实现“懒性初始化”（第一次 get_or_init 时才执行闭包）
// 其真正的优势在于：它允许 全局变量 的 初始化被推迟到 “第一次使用”的时候，
// 并且保证这个初始化过程是线程安全的（安全地初始化 任何对象(T)）。
// 关键词：全局变量，延迟初始化，单例

// 不同于 “Arc”：
// 其解决的是 一份 堆内存数据 如何在多个线程之间共享所有权
// 即：一对多数据对象(T)共享并计数 —— 共享所有权
// 关键词：多线程，共同拥有， 堆内存数据

// 共同点：为了解决 “多线程并发访问” 时的资源管理问题


// Mutex 保证“内部可变性”（允许多线程安全修改）
static COUNTER: OnceLock<Mutex<i32>> = OnceLock::new();

// 对应 “OnceLock<Mutex<i32>>” 的辅助函数：获取全局计数器的引用
fn get_counter() -> &'static Mutex<i32> {       // 返回值是一个具有 'static 生命周期的共享引用
    // &：借用/引用。
    //
    // 'static：生命周期修饰符，表示该引用在整个程序运行期间有效。
    //
    // Mutex<i32>：内部可变性包装器，用于线程安全的互斥锁。

    COUNTER.get_or_init(|| Mutex::new(0))      // 如果已经初始化，直接返回引用；如果没有，则执行闭包进行初始化
}

fn main() {

    // 2. 初始化前状态检查
    // OnceLock::get() 初始化前返回 None, is_some() 则为false
    println!("初始化前： {}", COUNTER.get().is_some());

    // 3. 创建 2个线程
    let handle1 = thread::spawn(|| {
        let mut num = get_counter().lock().unwrap();
        // num 是一个 MutexGuard<i32> 类型
        *num += 1;
    });

    let handle2 = thread::spawn(|| {
        let mut num = get_counter().lock().unwrap();
        *num += 1;
    });

    handle1.join().unwrap(); // join(): 让主线程暂停，等待子线程执行完毕
    handle2.join().unwrap();

    println!("初始化后： {}", COUNTER.get().is_some());

    let final_val = get_counter().lock().unwrap();
    println!("counter = {}", final_val);

}
