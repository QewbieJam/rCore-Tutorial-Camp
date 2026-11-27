use std::cell::RefCell;

struct MyCell<T> { inner: RefCell<T> }

impl<T> MyCell<T> {
    fn new(v: T) -> Self {
        Self { inner: RefCell::new(v) }
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.borrow_mut(); // 拿到 RefMut（可变守卫）
        f(&mut *guard)                                     // 把 &mut T 交给闭包
    }

    fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.inner.borrow();            // 拿到 Ref（只读守卫）
        f(&*guard)                                         // 把 &T 交给闭包
    }
}

fn main() {
    let c = MyCell::new(10);
    let r = c.with_mut(|x| { *x += 5; *x });
    println!("with_mut 返回: {}", r);
    println!("with_ref 读: {}", c.with_ref(|x| *x));
}
