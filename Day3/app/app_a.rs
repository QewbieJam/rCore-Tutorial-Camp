#![no_std]
#![no_main]
#![allow(missing_docs)]

include!("mini.rs");

#[no_mangle]
pub extern "C" fn app_main() -> i32 {
    let mut ti = TaskInfo {
        tid: 0,
        pid: 0,
        comm: [0u8; 16],
    };

    print("A: hello\n");
    sys_get_taskinfo(&mut ti);
    print("A: tid = "); print_u(ti.tid as u64);
    print("\n");
    print("A: pid = "); print_u(ti.pid as u64);
    print("\n");
    print("A: comm = "); print_comm(&ti.comm);
    print("\n");

    sys_getpid();
    print("A Status: Finished\n");
    0
}