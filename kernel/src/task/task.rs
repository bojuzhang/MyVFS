use crate::fs::{self, DynInode, FdTable, FileHandle, FileType, FsError, FsResult, OpenFlags};
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
    pub cwd: Mutex<Option<DynInode>>,
}

impl TaskControlBlock {
    pub fn new(pid: PidHandle) -> Self {
        Self {
            pid,
            task_status: Mutex::new(TaskStatus::Ready),
            fd_table: Mutex::new(fd_table_with_stdio()),
            cwd: Mutex::new(None),
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

    pub fn cwd(&self) -> FsResult<DynInode> {
        let mut cwd = self.cwd.lock().map_err(|_| FsError::Eio)?;
        if cwd.is_none() {
            *cwd = Some(fs::root_inode()?);
        }
        cwd.as_ref().cloned().ok_or(FsError::Eio)
    }

    pub fn set_cwd(&self, cwd: DynInode) -> FsResult<()> {
        if cwd.metadata()?.file_type != FileType::Directory {
            return Err(FsError::Enotdir);
        }
        *self.cwd.lock().map_err(|_| FsError::Eio)? = Some(cwd);
        Ok(())
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
