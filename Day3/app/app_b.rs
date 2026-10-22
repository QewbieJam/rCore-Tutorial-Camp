#![no_std]
#![no_main]
#![allow(missing_docs)]

include!("mini.rs");

fn show(nr: u32, name: &str) {
    let c = sys_get_syscall_count(nr);
    print(name);
    print("(");
    print_u(nr as u64);
    print(") = ");
    print_u(c as u64);
    print("\n");
}

#[no_mangle]
pub extern "C" fn app_main() -> i32 {
    let mut ti = TaskInfo {
        tid: 0,
        pid: 0,
        comm: [0u8; 16],
    };

    let r = sys_get_taskinfo(&mut ti);
    print("B: ret = ");
    print_u(r as u64);
    print("\n");
    print("tid =");
    print_u(ti.tid as u64);
    print("\n");
    print("pid =");
    print_u(ti.pid as u64);
    print("\n");
    print("comm =");
    print_comm(&ti.comm);
    print("\n");

    print("--- syscall counts ---");
    print("\n");
    show(1, "write");
    show(39,"getpid");
    show(60,"exit");
    show(463,"get_task_info");
    show(464,"get_syscall_count");
    show(465,"reset_syscall_counts");
    0
}