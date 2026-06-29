use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use std::thread::{self, Thread};

use crate::sync::{Mutex, MutexGuard};
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};

pub struct WaitQueue {
    gate: Mutex<()>,
    waiters: Mutex<VecDeque<Waiter>>,
}

struct Waiter {
    task: Option<Arc<TaskControlBlock>>,
    thread: Thread,
}

impl WaitQueue {
    pub fn new() -> Self {
        Self {
            gate: Mutex::new(()),
            waiters: Mutex::new(VecDeque::new()),
        }
    }

    pub fn prepare_wait(&self) -> MutexGuard<'_, ()> {
        self.gate.lock().expect("wait queue gate poisoned")
    }

    pub fn sleep_current(&self) {
        let guard = self.prepare_wait();
        self.sleep_current_with_guard(guard);
    }

    pub fn sleep_current_with_guard(&self, guard: MutexGuard<'_, ()>) {
        let task = current_task();
        let has_task = task.is_some();
        {
            let mut waiters = self.waiters.lock().expect("wait queue poisoned");
            waiters.push_back(Waiter {
                task,
                thread: thread::current(),
            });
        }
        drop(guard);
        if has_task {
            block_current_and_run_next();
        }
        thread::park();
    }

    pub fn wake_one(&self) {
        let guard = self.prepare_wait();
        let waiter = self
            .waiters
            .lock()
            .ok()
            .and_then(|mut waiters| waiters.pop_front());
        drop(guard);
        wake_waiter(waiter);
    }

    pub fn try_wake_one(&self) {
        self.wake_one();
    }

    pub fn wake_all(&self) {
        let guard = self.prepare_wait();
        let waiters = self
            .waiters
            .lock()
            .map(|mut waiters| waiters.drain(..).collect::<Vec<_>>())
            .unwrap_or_default();
        drop(guard);
        for waiter in waiters {
            wake_waiter(Some(waiter));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.waiters
            .lock()
            .map(|waiters| waiters.is_empty())
            .unwrap_or(true)
    }
}

impl fmt::Debug for WaitQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.waiters.try_lock() {
            Ok(waiters) => f
                .debug_struct("WaitQueue")
                .field("waiters", &waiters.len())
                .finish(),
            Err(_) => f
                .debug_struct("WaitQueue")
                .field("waiters", &"<locked>")
                .finish(),
        }
    }
}

impl Default for WaitQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn wake_waiter(waiter: Option<Waiter>) {
    if let Some(waiter) = waiter {
        if let Some(task) = waiter.task {
            wakeup_task(task);
        }
        waiter.thread.unpark();
    }
}
