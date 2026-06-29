# src/fs/packetfs/api.rs

## 文件职责

提供 `packetfs` 对外稳定 API。VFS 初始化通过这里创建文件系统，virtio-net RX 路径通过这里提交 Ethernet frame。

## 需要定义的 struct

```rust
pub struct RxMeta {
    pub timestamp_us: u64,
    pub iface_id: u32,
}
```

`iface_id` 先用于统计和调试，不要求支持多网卡。

## 需要定义的 enum

```rust
pub enum SubmitResult {
    Queued,
    DroppedInactive,
    DroppedFull,
    Truncated,
}
```

## 需要实现的接口

```rust
pub fn make_packetfs(config: PacketFsConfig) -> FsResult<Arc<dyn FileSystem>>;
pub fn submit_rx_frame(frame: &[u8], meta: RxMeta) -> SubmitResult;
pub fn stats_snapshot() -> FsResult<StatsSnapshot>;
```

可选：

```rust
pub fn set_active_instance(inner: Arc<PacketFsInner>) -> FsResult<()>;
pub fn clear_active_instance();
```

## 维护的信息

可以维护一个全局当前实例：

```rust
static ACTIVE_PACKETFS: OnceCell<Arc<PacketFsInner>>;
```

如果没有 `OnceCell`，可用 `UPSafeCell<Option<Arc<PacketFsInner>>>`。

## submit_rx_frame 流程

1. 查找当前活跃 `PacketFsInner`。
2. 如果没有挂载，计入或返回 `DroppedInactive`。
3. 如果 `mounted=false`，返回 `DroppedInactive`。
4. 获取 ring 锁。
5. 调用 `PacketRing::push_frame()`。
6. 更新 stats。
7. 入队成功时 `wait_queue.wake_one()`。
8. 返回 `SubmitResult`。

## 关键约束

- 不睡眠。
- 不访问用户缓冲区。
- 不做串口打印。
- 不长时间持锁。
- 不解析 IP/TCP。
