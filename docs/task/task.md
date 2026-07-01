# src/task/task.rs

## 额外说明

照搬 rCore，少量适配。

## 文件职责

定义 `TaskControlBlock` 和任务状态。VFS 通过当前任务取得 fd table 和 cwd。

## 需要确认的字段

```rust
pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub task_status: Mutex<TaskStatus>,
    pub fd_table: Mutex<FdTable>,
    pub cwd: Mutex<Option<DynInode>>,
    ...
}
```

字段名可按工程实际调整。

这里的 `Mutex` 是 `crate::sync::Mutex`，用于避免任务状态、fd table 和 cwd 继续依赖标准库锁。

## cwd 设计

- cwd 是进程属性，由 `TaskControlBlock` 持有，不由 `PathResolver` 自己维护。
- `TaskControlBlock::cwd()` 首次使用时以 VFS root 作为默认 cwd，并把该值保存在任务中。
- `TaskControlBlock::set_cwd()` 只接受目录 inode。
- `fs::resolver()` 每次解析路径时从 `current_task()` 获取 cwd 快照，然后构造 `PathResolver`。

## packetfs 相关适配

- `sys_open` 需要向当前任务 fd table 插入 file。
- `sys_read` 需要从当前任务 fd table 取 file。
- 相对路径解析需要使用当前任务 cwd。
- 阻塞 read 时当前任务状态应改成 blocked。
