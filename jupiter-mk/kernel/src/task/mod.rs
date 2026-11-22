//! Tasks, kernel stacks and the round-robin scheduler.
//!
//! The scheduler is deliberately tiny: single hart, no priorities, no
//! preemption while in S-mode. Timer interrupts arrive from U-mode only, which
//! keeps trap handling non-reentrant.

mod context;
mod ipc;

use alloc::alloc::{Layout, alloc_zeroed, dealloc};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::{asm, global_asm};
use core::mem::size_of;

use riscv::register::satp;
use spin::Mutex;

use crate::board;
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE};
use crate::ipc::{Cap, Endpoint, MAX_CAPS, Message};
use crate::mm::{MemorySet, USER_STACK_TOP, kernel_token, loader, page_table::PteFlags};
use crate::trap::TrapContext;
use crate::user_images;

pub use context::TaskContext;
pub use ipc::{recv, send, try_recv, try_send};

global_asm!(include_str!("switch.S"));

unsafe extern "C" {
    fn __switch(current: *mut TaskContext, next: *const TaskContext);
    fn __task_entry();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    BlockedSend(usize),
    BlockedRecv(usize),
    Zombie,
}

pub struct KernelStack {
    ptr: usize,
    layout: Layout,
}

impl KernelStack {
    fn new() -> Self {
        let layout = Layout::from_size_align(KERNEL_STACK_SIZE, 16).expect("invalid stack layout");
        let ptr = unsafe { alloc_zeroed(layout) };
        assert!(!ptr.is_null(), "out of memory: kernel stack");
        Self {
            ptr: ptr as usize,
            layout,
        }
    }

    fn top(&self) -> usize {
        self.ptr + self.layout.size()
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr as *mut u8, self.layout) };
    }
}

pub struct Task {
    pub pid: usize,
    pub name: &'static str,
    pub state: TaskState,
    pub ctx: TaskContext,
    /// Owns the kernel stack memory. Taken (and freed) when a zombie is
    /// reaped.
    pub kstack: Option<KernelStack>,
    /// User address space. Taken (and freed) when a zombie is reaped.
    pub memory_set: Option<MemorySet>,
    pub caps: [Option<Cap>; MAX_CAPS],
    pub ipc_msg: Message,
    pub ipc_result: isize,
    pub exit_code: isize,
}

impl Task {
    fn new(pid: usize, image_id: usize, caps: [Option<Cap>; MAX_CAPS]) -> Self {
        let image = user_images::image(image_id).expect("unknown user image");
        let mut memory_set = MemorySet::new_user();

        // The console server is the only task that owns the UART page. This
        // is the MVP version of a device capability.
        if image_id == user_images::CONSOLE {
            memory_set.map_identity_user(
                board::BOARD.uart_base,
                board::BOARD.uart_base + PAGE_SIZE,
                PteFlags::R | PteFlags::W | PteFlags::U,
            );
        }

        let entry = loader::load_elf(&mut memory_set, image);
        memory_set.map_user_stack();

        let kstack = KernelStack::new();
        let trap_cx_ptr = (kstack.top() - size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            trap_cx_ptr.write(TrapContext::new_user(entry, USER_STACK_TOP));
        }
        let ctx = TaskContext::new(__task_entry as *const () as usize, kstack.top());

        Self {
            pid,
            name: user_images::name(image_id),
            state: TaskState::Ready,
            ctx,
            kstack: Some(kstack),
            memory_set: Some(memory_set),
            caps,
            ipc_msg: Message::default(),
            ipc_result: 0,
            exit_code: 0,
        }
    }
}

pub struct TaskManager {
    pub tasks: Vec<Box<Task>>,
    pub current: Option<usize>,
    pub endpoints: Vec<Endpoint>,
    pub boot_caps: [Option<Cap>; MAX_CAPS],
    idle: TaskContext,
    next_pid: usize,
    rr: usize,
}

impl TaskManager {
    const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current: None,
            endpoints: Vec::new(),
            boot_caps: [None; MAX_CAPS],
            idle: TaskContext::zero(),
            next_pid: 1,
            rr: 0,
        }
    }

    fn pick_next(&mut self) -> Option<usize> {
        let n = self.tasks.len();
        for i in 0..n {
            let idx = (self.rr + i) % n;
            if self.tasks[idx].state == TaskState::Ready {
                self.rr = (idx + 1) % n;
                return Some(idx);
            }
        }
        None
    }

    /// Free the heavy resources of tasks that exited but are not the current
    /// task. The current task's kernel stack is still in use until `__switch`
    /// has switched away, so it is reaped on a later scheduling pass.
    fn reap_zombies(&mut self, current: Option<usize>) {
        for (i, task) in self.tasks.iter_mut().enumerate() {
            if Some(i) == current || task.state != TaskState::Zombie {
                continue;
            }
            let memory_set = task.memory_set.take();
            let kstack = task.kstack.take();
            drop(memory_set);
            drop(kstack);
        }
    }
}

pub static TASKS: Mutex<TaskManager> = Mutex::new(TaskManager::new());

pub fn init() {
    {
        let mut m = TASKS.lock();
        m.endpoints.push(Endpoint::new()); // 0: console requests
        m.endpoints.push(Endpoint::new()); // 1: console input
        m.boot_caps[0] = Some(Cap::Endpoint(0));
        m.boot_caps[1] = Some(Cap::Endpoint(1));
    }

    spawn_boot(user_images::CONSOLE);
    spawn_boot(user_images::SHELL);

    let m = TASKS.lock();
    println!(
        "[task] {} tasks, endpoints {}",
        m.tasks.len(),
        m.endpoints.len()
    );
    for task in &m.tasks {
        println!(
            "  pid {} {:<8} entry={:#x}",
            task.pid, task.name, task.ctx.ra
        );
    }
}

fn spawn_boot(image_id: usize) -> usize {
    let caps = TASKS.lock().boot_caps;
    spawn_with_caps(image_id, caps)
}

pub fn spawn(image_id: usize) -> isize {
    let caps = {
        let m = TASKS.lock();
        match m.current {
            Some(i) => m.tasks[i].caps,
            None => m.boot_caps,
        }
    };
    spawn_with_caps(image_id, caps) as isize
}

fn spawn_with_caps(image_id: usize, caps: [Option<Cap>; MAX_CAPS]) -> usize {
    let (index, pid) = {
        let mut m = TASKS.lock();
        let index = m.tasks.len();
        let pid = m.next_pid;
        m.next_pid += 1;
        (index, pid)
    };

    let task = Task::new(pid, image_id, caps);

    let mut m = TASKS.lock();
    assert_eq!(m.tasks.len(), index, "task table changed during spawn");
    m.tasks.push(Box::new(task));
    pid
}

pub fn current_pid() -> usize {
    let m = TASKS.lock();
    m.current.map(|i| m.tasks[i].pid).unwrap_or(0)
}

/// Save the current task's kernel context and switch to the next runnable
/// task. If none is runnable, switch back to the boot/idle context.
pub fn schedule() {
    let (prev, next, token) = {
        let mut m = TASKS.lock();
        let prev = m.current;
        m.reap_zombies(prev);
        match m.pick_next() {
            Some(n) => {
                m.tasks[n].state = TaskState::Running;
                let prev_ptr = match prev {
                    Some(p) => core::ptr::addr_of_mut!(m.tasks[p].ctx),
                    None => core::ptr::addr_of_mut!(m.idle),
                };
                let next_ptr = core::ptr::addr_of!(m.tasks[n].ctx);
                let token = m.tasks[n]
                    .memory_set
                    .as_ref()
                    .expect("runnable task without address space")
                    .token();
                m.current = Some(n);
                (prev_ptr, next_ptr, token)
            }
            None => {
                let prev_idx = prev.expect("deadlock: no runnable tasks");
                let prev_ptr = core::ptr::addr_of_mut!(m.tasks[prev_idx].ctx);
                let next_ptr = core::ptr::addr_of!(m.idle);
                m.current = None;
                (prev_ptr, next_ptr, kernel_token())
            }
        }
    };

    unsafe {
        satp::write(satp::Satp::from_bits(token));
        asm!("sfence.vma");
        __switch(prev, next);
    }
}

/// Called on a timer interrupt: put the current task back on the ready queue.
pub fn preempt_current() {
    {
        let mut m = TASKS.lock();
        if let Some(i) = m.current {
            if m.tasks[i].state == TaskState::Running {
                m.tasks[i].state = TaskState::Ready;
            }
        }
    }
    schedule();
}

pub fn exit_current(code: isize) -> ! {
    {
        let mut m = TASKS.lock();
        if let Some(i) = m.current {
            m.tasks[i].state = TaskState::Zombie;
            m.tasks[i].exit_code = code;
            println!(
                "[task] pid {} ({}) exited with {}",
                m.tasks[i].pid, m.tasks[i].name, code
            );
        }
    }
    loop {
        schedule();
    }
}

pub fn run_tasks() -> ! {
    schedule();
    loop {
        schedule();
    }
}

/// Copy a user message from user virtual memory using the page table, not
/// SUM. Returns `None` for an unmapped pointer.
pub fn read_user_message(ptr: usize) -> Option<Message> {
    if ptr % core::mem::align_of::<usize>() != 0 {
        return None;
    }
    let m = TASKS.lock();
    let cur = m.current?;
    let space = m.tasks[cur]
        .memory_set
        .as_ref()
        .expect("current task has no address space");
    let mut words = [0usize; 1 + crate::ipc::MSG_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        let pa = space.translate_va(ptr + i * 8)?;
        *word = unsafe { *(pa as *const usize) };
    }
    Some(Message {
        tag: words[0],
        words: [words[1], words[2], words[3], words[4]],
    })
}

pub fn write_user_message(ptr: usize, msg: &Message) -> bool {
    if ptr % core::mem::align_of::<usize>() != 0 {
        return false;
    }
    let m = TASKS.lock();
    let cur = match m.current {
        Some(c) => c,
        None => return false,
    };
    let space = m.tasks[cur]
        .memory_set
        .as_ref()
        .expect("current task has no address space");
    let words = [
        msg.tag,
        msg.words[0],
        msg.words[1],
        msg.words[2],
        msg.words[3],
    ];
    for (i, word) in words.iter().enumerate() {
        let pa = match space.translate_va(ptr + i * 8) {
            Some(pa) => pa,
            None => return false,
        };
        unsafe {
            *(pa as *mut usize) = *word;
        }
    }
    true
}

/// A slice for the temporary debug-write syscall.
pub fn read_user_bytes(ptr: usize, len: usize, out: &mut [u8]) -> bool {
    if len > out.len() {
        return false;
    }
    let m = TASKS.lock();
    let cur = match m.current {
        Some(c) => c,
        None => return false,
    };
    let space = m.tasks[cur]
        .memory_set
        .as_ref()
        .expect("current task has no address space");
    for (i, byte) in out.iter_mut().enumerate().take(len) {
        let pa = match space.translate_va(ptr + i) {
            Some(pa) => pa,
            None => return false,
        };
        *byte = unsafe { *(pa as *const u8) };
    }
    true
}
