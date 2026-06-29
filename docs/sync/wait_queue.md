# src/sync/wait_queue.rs

## 额外说明

新增或参考 rCore 改写。

## 文件职责

提供阻塞与唤醒机制。`/packets` 在无包时阻塞当前任务，`submit_rx_frame()` 入队后唤醒 reader。

## 需要定义的 struct

```rust
pub struct WaitQueue {
    pub waiters: VecDeque<Arc<TaskControlBlock>>,
}
```

实际实现需要由锁保护。

## 需要实现的接口

```rust
impl WaitQueue {
    pub fn new() -> Self;
    pub fn sleep_current(&self);
    pub fn wake_one(&self);
    pub fn wake_all(&self);
    pub fn is_empty(&self) -> bool;
}
```

## 核心流程

`sleep_current()`：

1. 获取当前任务。
2. 将当前任务加入 waiters。
3. 将任务状态改为 blocked。
4. 调用 `block_current_and_run_next()`。

`wake_one()`：

1. 从 waiters 弹出一个任务。
2. 调用 `wakeup_task(task)`。

`wake_all()`：

1. 弹出所有任务。
2. 逐个唤醒。

## packetfs 使用场景

- `/packets` 空队列 read：`sleep_current()`。
- submit 入队成功：`wake_one()`。
- umount：`wake_all()`。

## 错误与边界

- 睡眠前必须在循环中重新检查条件，避免 missed wakeup。
- 被唤醒后必须重新检查 ring 是否非空。
- umount 唤醒后 read 应返回 `EIO`。
