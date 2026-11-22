//! Synchronous endpoint IPC.
//!
//! `send` blocks until a receiver appears; `recv` blocks until a sender
//! appears; `try_recv` is the non-blocking variant used by the console server
//! so that it can also poll the UART.

use super::{TASKS, TaskState, schedule};
use crate::ipc::{Cap, Message};

fn endpoint_of(slot: usize) -> Option<usize> {
    let m = TASKS.lock();
    let cur = m.current?;
    match m.tasks[cur].caps.get(slot).copied().flatten() {
        Some(Cap::Endpoint(ep)) if ep < m.endpoints.len() => Some(ep),
        _ => None,
    }
}

pub fn send(slot: usize, msg: Message) -> isize {
    let ep = match endpoint_of(slot) {
        Some(ep) => ep,
        None => return -1,
    };

    let cur;
    {
        let mut m = TASKS.lock();
        cur = m.current.expect("ipc_send from idle");

        // Fast path: a receiver is already waiting.
        if let Some(ridx) = m.endpoints[ep].receivers.pop_front() {
            m.tasks[ridx].ipc_msg = msg;
            m.tasks[ridx].ipc_result = 0;
            m.tasks[ridx].state = TaskState::Ready;
            return 0;
        }

        m.tasks[cur].state = TaskState::BlockedSend(ep);
        m.tasks[cur].ipc_msg = msg;
        m.endpoints[ep].senders.push_back(cur);
    }

    schedule();

    let m = TASKS.lock();
    m.tasks[cur].ipc_result
}

/// Non-blocking send: drop the message if no receiver is waiting.
pub fn try_send(slot: usize, msg: Message) -> isize {
    let ep = match endpoint_of(slot) {
        Some(ep) => ep,
        None => return -1,
    };
    let mut m = TASKS.lock();
    if let Some(ridx) = m.endpoints[ep].receivers.pop_front() {
        m.tasks[ridx].ipc_msg = msg;
        m.tasks[ridx].ipc_result = 0;
        m.tasks[ridx].state = TaskState::Ready;
        0
    } else {
        -1
    }
}

pub fn recv(slot: usize) -> Option<Message> {
    recv_inner(slot, true)
}

pub fn try_recv(slot: usize) -> Option<Message> {
    recv_inner(slot, false)
}

fn recv_inner(slot: usize, block: bool) -> Option<Message> {
    let ep = endpoint_of(slot)?;

    let cur;
    {
        let mut m = TASKS.lock();
        cur = m.current.expect("ipc_recv from idle");

        // Fast path: a sender is already waiting.
        if let Some(sidx) = m.endpoints[ep].senders.pop_front() {
            let msg = m.tasks[sidx].ipc_msg;
            m.tasks[sidx].ipc_result = 0;
            m.tasks[sidx].state = TaskState::Ready;
            return Some(msg);
        }

        if !block {
            return None;
        }

        m.tasks[cur].state = TaskState::BlockedRecv(ep);
        m.endpoints[ep].receivers.push_back(cur);
    }

    schedule();

    let m = TASKS.lock();
    Some(m.tasks[cur].ipc_msg)
}
