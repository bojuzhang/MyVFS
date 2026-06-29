use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};

use crate::sync::Mutex;
use crate::task::TaskControlBlock;

static READY_QUEUE: OnceLock<Mutex<VecDeque<Arc<TaskControlBlock>>>> = OnceLock::new();

pub fn add_task(task: Arc<TaskControlBlock>) {
    let queue = READY_QUEUE.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Ok(mut queue) = queue.lock() {
        queue.push_back(task);
    }
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    READY_QUEUE
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .ok()
        .and_then(|mut queue| queue.pop_front())
}
