# src/sync/mutex.rs

## 额外说明

参考 rCore 的自写锁思路，当前实现不再包装 `std::sync::Mutex`。

## 文件职责

提供内核共享状态使用的自旋互斥锁。项目当前允许其他 `std` 能力继续存在，但锁必须来自这里。

## 当前实现

```rust
pub type Mutex<T> = SpinMutex<T>;

pub struct SpinMutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}
```

- `lock()` 自旋等待并返回 `MutexGuard`。
- `try_lock()` 忙时返回 `TryLockError::WouldBlock`。
- 不维护 poisoning；保留 `LockResult`/`TryLockError::Poisoned` 形状是为了让现有调用点少改动。
- `MutexGuard` 通过 `Drop` 释放锁。

## packetfs 需要保护的状态

- `PacketRing`
- `PacketStatsInner`
- `StatsFile` offset
- `PacketCaptureFile` current buffer

## 需要说明的类型

```rust
pub type Mutex<T> = SpinMutex<T>;
pub struct MutexGuard<'a, T: ?Sized>;
pub trait Lock<T: ?Sized>;
```

## 选择建议

`submit_rx_frame()` 可能在网卡轮询或中断上下文调用。ring 锁应避免睡眠，当前用 `try_lock()` 在忙时计入捕获侧丢包；后续如果引入真实中断上下文，可再升级为关中断锁。
