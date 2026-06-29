use crate::fs::FdTable;
use crate::sync::Mutex;
use crate::task::id::PidHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
    Exited,
}

pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub task_status: Mutex<TaskStatus>,
    pub fd_table: Mutex<FdTable>,
}

impl TaskControlBlock {
    pub fn new(pid: PidHandle) -> Self {
        Self {
            pid,
            task_status: Mutex::new(TaskStatus::Ready),
            fd_table: Mutex::new(FdTable::new()),
        }
    }

    pub fn set_status(&self, status: TaskStatus) {
        if let Ok(mut guard) = self.task_status.lock() {
            *guard = status;
        }
    }

    pub fn status(&self) -> TaskStatus {
        self.task_status
            .lock()
            .map(|guard| *guard)
            .unwrap_or(TaskStatus::Exited)
    }
}
