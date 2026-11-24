use std::collections::VecDeque;
use crate::State::Ready;

#[derive(Debug)]
enum State { Ready, Running, Exited }

#[derive(Debug)]
struct Pcb {
    pid: u32,
    state: State,
    regs: [u64; 32],
    sepc: u64,
    stack: Vec<u64>,
}

struct Scheduler {
    ready: VecDeque<Pcb>,
    current: Option<Pcb>,
    next_pid: u32,
}

impl Scheduler {
    fn spawn(&mut self) {
        let pid = self.next_pid;

        let pcb = Pcb {
            pid,
            state: State::Ready,
            regs: [0; 32],
            sepc: 0,
            stack: Vec::new(),
        };
        self.ready.push_back(pcb);
        self.next_pid += 1;
        println!("spawned {} pcb, state: Ready", pid);
    }
}


fn main() {

    let mut s = Scheduler {
        ready: VecDeque::new(),
        current: None,
        next_pid: 1,
    };

    s.spawn();
    s.spawn();
    s.spawn();

    println!("ready 里有 {} 个", s.ready.len());
}
