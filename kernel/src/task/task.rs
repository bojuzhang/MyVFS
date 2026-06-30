use crate::fs::{self, FdTable, FileHandle, OpenFlags};
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
            fd_table: Mutex::new(fd_table_with_stdio()),
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

fn fd_table_with_stdio() -> FdTable {
    let mut table = FdTable::new();
    let _ = table.alloc(FileHandle {
        file: fs::stdin(),
        flags: OpenFlags::RDONLY,
        offset: 0,
        debug_path: "stdin".to_string(),
    });
    let _ = table.alloc(FileHandle {
        file: fs::stdout(),
        flags: OpenFlags::WRONLY,
        offset: 0,
        debug_path: "stdout".to_string(),
    });
    let _ = table.alloc(FileHandle {
        file: fs::stdout(),
        flags: OpenFlags::WRONLY,
        offset: 0,
        debug_path: "stderr".to_string(),
    });
    table
}
