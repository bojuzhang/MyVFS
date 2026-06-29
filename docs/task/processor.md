# src/task/processor.rs

## 额外说明

照搬 rCore，少量适配。

## 文件职责

维护当前 CPU 正在运行的任务，并提供让出、阻塞、唤醒相关接口。

## packetfs 需要的接口

```rust
pub fn current_task() -> Option<Arc<TaskControlBlock>>;
pub fn current_user_token() -> usize;
pub fn block_current_and_run_next();
pub fn suspend_current_and_run_next();
pub fn wakeup_task(task: Arc<TaskControlBlock>);
```

## `/packets` 阻塞读使用方式

```text
PacketCaptureFile::read()
 -> WaitQueue::prepare_wait()
 -> WaitQueue::sleep_current_with_guard()
 -> block_current_and_run_next()
```

收包：

```text
submit_rx_frame()
 -> wait_queue.wake_one()
 -> wakeup_task(task)
```
