# src/task/task.rs

## 额外说明

照搬 rCore，少量适配。

## 文件职责

定义 `TaskControlBlock` 和任务状态。packetfs 只关心当前任务中是否能找到 fd table。

## 需要确认的字段

```rust
pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub task_status: Mutex<TaskStatus>,
    pub fd_table: Mutex<FdTable>,
    ...
}
```

字段名可按工程实际调整。

这里的 `Mutex` 是 `crate::sync::Mutex`，用于避免任务状态和 fd table 继续依赖标准库锁。

## packetfs 相关适配

- `sys_open` 需要向当前任务 fd table 插入 file。
- `sys_read` 需要从当前任务 fd table 取 file。
- 阻塞 read 时当前任务状态应改成 blocked。
