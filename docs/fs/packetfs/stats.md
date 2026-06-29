# src/fs/packetfs/stats.rs

## 分类

新增实现。

## 文件职责

维护 `packetfs` 的统计计数，并把计数渲染成 `/stats` 文件读取的文本。

## 需要定义的 struct

```rust
pub struct PacketStats {
    pub inner: Mutex<PacketStatsInner>,
}
```

这里的 `Mutex` 是 `crate::sync::Mutex`。收包侧的 `try_*` 更新接口使用 `try_lock()`，忙时允许跳过统计更新以避免在 RX 路径睡眠。

```rust
pub struct PacketStatsInner {
    pub captured_packets: u64,
    pub captured_bytes: u64,
    pub read_packets: u64,
    pub read_bytes: u64,
    pub queued_packets: u64,
    pub dropped_full: u64,
    pub dropped_inactive: u64,
    pub truncated_packets: u64,
    pub reader_active: bool,
    pub last_rx_ts: u64,
}
```

```rust
pub struct StatsSnapshot {
    pub captured_packets: u64,
    pub captured_bytes: u64,
    pub read_packets: u64,
    pub read_bytes: u64,
    pub queued_packets: u64,
    pub dropped_full: u64,
    pub dropped_inactive: u64,
    pub truncated_packets: u64,
    pub reader_active: bool,
    pub last_rx_ts: u64,
}
```

## 需要实现的接口

```rust
impl PacketStats {
    pub fn new() -> Self;
    pub fn on_rx(&self, bytes: usize, timestamp_us: u64);
    pub fn try_on_rx(&self, bytes: usize, timestamp_us: u64) -> bool;
    pub fn on_read(&self, bytes: usize);
    pub fn on_drop_full(&self);
    pub fn try_on_drop_full(&self) -> bool;
    pub fn on_drop_inactive(&self);
    pub fn try_on_drop_inactive(&self) -> bool;
    pub fn on_truncate(&self);
    pub fn try_on_truncate(&self) -> bool;
    pub fn set_queued_packets(&self, queued: usize);
    pub fn try_set_queued_packets(&self, queued: usize) -> bool;
    pub fn set_reader_active(&self, active: bool);
    pub fn snapshot(&self) -> StatsSnapshot;
}
```

```rust
impl StatsSnapshot {
    pub fn render_text(&self) -> Vec<u8>;
}
```

## 文本格式

一行一个 key：

```text
filesystem=packetfs
mounted=true
captured_packets=10
captured_bytes=640
read_packets=7
read_bytes=512
queued_packets=3
dropped_full=0
dropped_inactive=0
truncated_packets=1
reader_active=true
last_rx_ts=12345678
```

## 维护的信息

- 抓到多少包。
- 读出多少包。
- 队列里还有多少包。
- 因队列满丢了多少新包。
- 未挂载时丢了多少包。
- 因 snaplen 截断了多少包。
- 当前是否有 reader。
