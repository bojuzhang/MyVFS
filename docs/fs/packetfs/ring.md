# src/fs/packetfs/ring.rs

## 文件职责

维护 `packetfs` 的有界 packet queue。virtio-net 收到包后入队，`/packets` reader 从队列消费包。

## 需要定义的 struct

```rust
pub struct PacketRecord {
    pub seq: u64,
    pub timestamp_us: u64,
    pub wire_len: usize,
    pub cap_len: usize,
    pub data: Vec<u8>,
}
```

```rust
pub struct PacketRing {
    pub queue: VecDeque<PacketRecord>,
    pub capacity: usize,
    pub next_seq: u64,
}
```

## 需要实现的接口

```rust
impl PacketRing {
    pub fn new(capacity: usize) -> Self;
    pub fn push_frame(
        &mut self,
        frame: &[u8],
        timestamp_us: u64,
        snaplen: usize,
    ) -> PushOutcome;
    pub fn pop_frame(&mut self) -> Option<PacketRecord>;
    pub fn len(&self) -> usize;
    pub fn is_full(&self) -> bool;
    pub fn clear(&mut self);
}
```

```rust
pub enum PushOutcome {
    Queued,
    DroppedFull,
    Truncated,
}
```

## 维护的信息

- 当前已排队 packet。
- 队列容量上限。
- 下一个 packet 序号。
- 每个 packet 的原始长度和捕获长度。

## 入队流程

`push_frame(frame, timestamp_us, snaplen)`：

1. 如果队列已满，丢弃新包，返回 `DroppedFull`。
2. 计算 `wire_len = frame.len()`。
3. 计算 `cap_len = min(wire_len, snaplen)`。
4. 复制 `frame[..cap_len]`。
5. 创建 `PacketRecord`。
6. `next_seq += 1`。
7. 入队。
8. 如果发生截断，返回 `Truncated`，否则返回 `Queued`。

## 出队流程

`pop_frame()`：

1. 从队首弹出最早 packet。
2. 返回 `PacketRecord`。
3. 如果空队列，返回 `None`。

## 为什么队列满丢新包

网卡接收路径不应该因为用户程序读得慢而阻塞。丢新包可以保留已排队 packet 的顺序，并通过 `dropped_full` 明确告诉用户发生捕获侧丢包。
