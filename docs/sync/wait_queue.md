# src/sync/wait_queue.rs

## 额外说明

参考 rCore 的任务阻塞/唤醒模型，并为当前宿主 `std` 运行方式保留 `thread::park/unpark` 兼容。

## 文件职责

提供阻塞与唤醒机制。`/packets` 在无包时阻塞当前任务，`submit_rx_frame()` 入队后唤醒 reader。

## 当前结构

```rust
pub struct WaitQueue {
    gate: Mutex<()>,
    waiters: Mutex<VecDeque<Waiter>>,
}
```

`gate` 用来保护“检查条件 -> 加入等待队列 -> 入睡”的窗口，避免收包路径在这个窗口里丢失唤醒。两个锁都来自 `crate::sync::Mutex`。

## 需要实现的接口

```rust
impl WaitQueue {
    pub fn new() -> Self;
    pub fn prepare_wait(&self) -> MutexGuard<'_, ()>;
    pub fn sleep_current(&self);
    pub fn sleep_current_with_guard(&self, guard: MutexGuard<'_, ()>);
    pub fn wake_one(&self);
    pub fn try_wake_one(&self);
    pub fn wake_all(&self);
    pub fn is_empty(&self) -> bool;
}
```

## 核心流程

普通 `sleep_current()`：

1. 获取 `gate`。
2. 调用 `sleep_current_with_guard()`。

`packetfs` 阻塞读使用 `prepare_wait()`：

1. 先获取 `gate`。
2. 在持有 `gate` 时重新检查 ring 和 mounted 状态。
3. 如果仍需等待，调用 `sleep_current_with_guard(guard)`。
4. `sleep_current_with_guard()` 将当前任务和宿主线程加入 waiters。
5. 释放 `gate`。
6. 将任务状态改为 blocked，调用 `block_current_and_run_next()`。
7. 当前宿主线程 `park()`，被唤醒后回到调用方循环重新检查条件。

`wake_one()`：

1. 获取 `gate`。
2. 从 waiters 弹出一个 waiter。
3. 释放 `gate`。
4. 如果 waiter 带有任务，调用 `wakeup_task(task)`。
5. `unpark()` 对应宿主线程。

`wake_all()`：

1. 弹出所有任务。
2. 逐个唤醒。

## packetfs 使用场景

- `/packets` 空队列 read：`prepare_wait()` 后调用 `sleep_current_with_guard()`。
- submit 入队成功：`wake_one()`。
- umount：`wake_all()`。

## 错误与边界

- 睡眠前必须在循环中重新检查条件，避免 missed wakeup。
- 被唤醒后必须重新检查 ring 是否非空。
- umount 唤醒后 read 应返回 `EIO`。
