// 自测全部里程碑：期望输出 6 / 55 / 120 / 11 / 7 / 30
fn add3(a, b, c) {
    return a + b + c;
}

fn fib(n) {
    if (n < 2) { return n; }
    return fib(n - 1) + fib(n - 2);
}

fn fact(n) {
    let r = 1;
    while (n > 1) {
        r = r * n;
        n = n - 1;
    }
    return r;
}

fn main() {
    print(add3(1, 2, 3));      // M3: 函数调用  -> 6
    print(fib(10));            // M4: 递归      -> 55
    print(fact(5));            // M5: while     -> 120

    let x = 3;
    print(x + 4 * 2);          // M2: 优先级    -> 11
    print(add3(x, 3, 1));      // 变量传参      -> 7

    let s = 0;
    let i = 1;
    while (i <= 10) {          // M5: 循环求和  -> 55
        s = s + i;
        i = i + 1;
    }
    print(s);

    if (x == 3) { print(1); } else { print(0); }
    return 0;
}
