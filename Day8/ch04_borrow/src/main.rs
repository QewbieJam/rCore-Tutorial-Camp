use std::thread;


fn main() {

    let mut v = vec![1, 2, 3];

    // let a = &v;

    let b = &mut v;

    b.push(4);


    let (left, right) = v.split_at_mut(2);
    left[0] = 10;
    right[0] = 10;

    // *** split_at_mut 自身签名如下： ***
    // fn split_at_mut(&mut self, mid: usize) -> (&mut [T], &mut [T])
    // *** split_at_mut 使用场景 ***
    // 在多线程里，把一个数组分给两个线程去处理

    // 单独的 thread::spawn 缺少 &'static 全局生命周期声明，需使用 scope
    // thread::spwan 创建的是 游离线程（Detached Thread）,
    // 需要搭配 JoinHandle 或者 .join()

    // thread::spawn(move || {
    //     left[0] = 100;
    // });
    // thread::spawn(move || {
    //     right[0] = 200;
    // });
    // println!("{:?}", left);   这里会报错



    // 在 thread::scope 内部实现 spawn
    thread::scope(|s| {
        s.spawn(|| {
            left[0] = 100;
        });
    });
    thread::scope(|s| {
        s.spawn(|| {
            right[0] = 200;
        });
    }); // 作用域结束：Rust 会在此处自动等待两个子线程执行完毕


    println!("left: {:?}", left);
    println!("v: {:?}", v);
}
