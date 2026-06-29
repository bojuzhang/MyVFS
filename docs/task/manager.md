# src/task/manager.rs

## 额外说明

照搬 rCore。

## 文件职责

维护 ready queue，提供任务加入和取出接口。

## packetfs 依赖点

当 `/packets` read 阻塞时，当前任务不应继续留在 ready queue。收到包后，等待队列会通过 processor/manager 接口把任务放回 ready queue。

## 需要说明的接口

```rust
pub fn add_task(task: Arc<TaskControlBlock>);
pub fn fetch_task() -> Option<Arc<TaskControlBlock>>;
```
