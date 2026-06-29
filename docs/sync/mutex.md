# src/sync/mutex.rs

## 额外说明

照搬 rCore，packetfs 使用。

## 文件职责

提供互斥锁或自旋锁，保护共享状态。

## packetfs 需要保护的状态

- `PacketRing`
- `PacketStatsInner`
- `StatsFile` offset
- `PacketCaptureFile` current buffer

## 需要说明的类型

```rust
pub trait Mutex<T> {
    fn lock(&self) -> MutexGuard<T>;
}
```

具体实现可以是：

- `SpinMutex`
- `UPSafeCell`
- `SpinNoIrqLock`

## 选择建议

`submit_rx_frame()` 可能在网卡轮询或中断上下文调用。ring 锁应避免睡眠，优先使用自旋锁或关中断锁。
