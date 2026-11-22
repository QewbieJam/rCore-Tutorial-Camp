//! IPC objects: endpoints, messages and capability handles.
//!
//! Delivery itself lives in `task::ipc`, because it needs to manipulate task
//! states and the scheduler.

use alloc::collections::VecDeque;

pub const MAX_CAPS: usize = 8;
pub const MSG_WORDS: usize = 4;

/// A typed capability. For the MVP the only object is an endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap {
    Endpoint(usize),
}

/// Fixed-size synchronous message.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Message {
    pub tag: usize,
    pub words: [usize; MSG_WORDS],
}

#[derive(Default)]
pub struct Endpoint {
    /// Task indices blocked in `send`, in arrival order.
    pub senders: VecDeque<usize>,
    /// Task indices blocked in `recv`, in arrival order.
    pub receivers: VecDeque<usize>,
}

impl Endpoint {
    pub fn new() -> Self {
        Self::default()
    }
}
