use std::collections::VecDeque;


#[derive(Debug)]
enum State { Ready, Running, Exited }

#[derive(Debug)]
struct Pcb {
    pid: u32,
    state: State,
    regs: [u64; 32],
    sepc: u64,
    stack: Vec<u64>,
}   // 一个 PCB 大小为 296 字节，各自段大小分别是 4B, 1B, (3B填充), 256B, 8B, 24B

struct Scheduler {
    ready: VecDeque<Pcb>,   // 就绪队列 采用 双端队列 VecDeque，由 std 库提供，两端都能进出 push_back（新任务入队尾） / push_front / pop_back / pop_front（从队首取任务）
    current: Option<Pcb>,
    next_pid: u32,
}   // 一个 Scheduler 大小为 336 字节，三个字段分别是 32B, 296B, 4B, (填充4B)，结构体大小必须是最大对齐的整数倍。

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
        self.ready.push_back(pcb);  // spawn：新进程进队尾
        self.next_pid += 1;
        println!("spawned {} pcb, state: Ready", pid);
    }

    fn schedule(&mut self) -> bool {

        if let Some(mut cur) = self.current.take() {
            cur.state = State::Ready;
            println!("pid = {}", cur.pid);
            self.ready.push_back(cur);
        }
        // ② 从队首取下一个任务
        match self.ready.pop_front() {
            Some(mut next) => {
                next.state = State::Running;
                println!("pid = {}",next.pid);
                self.current = Some(next);
                true
            }
            None => false
        }
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

    s.schedule();
    s.schedule();
}
