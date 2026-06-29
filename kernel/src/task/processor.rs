use std::sync::{Arc, OnceLock};

use crate::sync::Mutex;
use crate::task::{add_task, fetch_task, pid_alloc, TaskControlBlock, TaskStatus};

static BOOTSTRAP_TASK: OnceLock<Arc<TaskControlBlock>> = OnceLock::new();
static CURRENT_TASK: OnceLock<Mutex<Option<Arc<TaskControlBlock>>>> = OnceLock::new();

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    let current = CURRENT_TASK.get_or_init(|| Mutex::new(None));
    let mut guard = current.lock().ok()?;
    if let Some(task) = guard.as_ref() {
        return Some(task.clone());
    }

    let next = fetch_task().or_else(|| {
        let bootstrap = BOOTSTRAP_TASK.get_or_init(|| Arc::new(TaskControlBlock::new(pid_alloc())));
        (bootstrap.status() != TaskStatus::Blocked).then(|| bootstrap.clone())
    })?;
    next.set_status(TaskStatus::Running);
    *guard = Some(next.clone());
    Some(next)
}

pub fn current_user_token() -> usize {
    0
}

pub fn block_current_and_run_next() {
    let current = CURRENT_TASK.get_or_init(|| Mutex::new(None));
    let Ok(mut guard) = current.lock() else {
        return;
    };
    if guard.is_none() {
        drop(guard);
        let _ = current_task();
        guard = match current.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
    }
    if let Some(task) = guard.take() {
        task.set_status(TaskStatus::Blocked);
    }
    schedule_next_locked(&mut guard);
}

pub fn suspend_current_and_run_next() {
    let current = CURRENT_TASK.get_or_init(|| Mutex::new(None));
    let Ok(mut guard) = current.lock() else {
        return;
    };
    if guard.is_none() {
        drop(guard);
        let _ = current_task();
        guard = match current.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
    }
    if let Some(task) = guard.take() {
        task.set_status(TaskStatus::Ready);
        add_task(task);
    }
    schedule_next_locked(&mut guard);
}

pub fn wakeup_task(task: Arc<TaskControlBlock>) {
    task.set_status(TaskStatus::Ready);
    add_task(task);
}

fn schedule_next_locked(current: &mut Option<Arc<TaskControlBlock>>) {
    if let Some(task) = fetch_task() {
        task.set_status(TaskStatus::Running);
        *current = Some(task);
    }
}
