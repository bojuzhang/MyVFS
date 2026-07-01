# src/fs/packetfs/fs.rs

## 文件职责

定义 `packetfs` 文件系统类型、挂载配置、挂载实例共享状态。它相当于 Linux 文件系统里的 superblock 状态承载处。

## 需要定义的 struct

```rust
pub struct PacketFsConfig {
    pub snaplen: usize,
    pub capacity: usize,
}
```

字段解释：

- `snaplen`：每个包最多保存多少字节。
- `capacity`：ring 最多缓存多少个包。

默认：

```text
snaplen = 2048
capacity = 256
```

限制：

```text
64 <= snaplen <= 4096
1 <= capacity <= 4096
```

```rust
pub struct PacketFs {
    pub inner: Arc<PacketFsInner>,
}
```

```rust
pub struct PacketFsInner {
    pub config: PacketFsConfig,
    pub root_inode: Arc<PacketInode>,
    pub packets_inode: Arc<PacketInode>,
    pub stats_inode: Arc<PacketInode>,
    pub ring: Mutex<PacketRing>,
    pub stats: PacketStats,
    pub wait_queue: WaitQueue,
    pub reader_active: AtomicBool,
    pub mounted: AtomicBool,
}
```

这里的 `Mutex` 和 `WaitQueue` 都来自 `crate::sync`。`fs.rs` 不再定义局部 `Condvar` 等待队列，也不使用 `std::sync::Mutex`。

## 需要实现的 impl

```rust
impl PacketFsConfig {
    pub fn default() -> Self;
    pub fn parse(options: &str) -> FsResult<Self>;
    pub fn validate(&self) -> FsResult<()>;
}
```

```rust
impl PacketFs {
    pub fn new(config: PacketFsConfig) -> FsResult<Self>;
}
```

```rust
impl FileSystem for PacketFs {
    fn name(&self) -> &'static str;
    fn mount(&self, options: &str) -> FsResult<DynInode>;
    fn umount(&self) -> FsResult<()>;
    fn root_inode(&self) -> DynInode;
}
```

## 维护的信息

- 挂载配置。
- 三个固定 inode。
- packet queue。
- packet stats。
- 阻塞读等待队列，使用 `sync::WaitQueue`。
- 单读者状态。
- 是否仍处于 mounted 状态。

## 核心流程

`PacketFs::mount(options)`：

1. 解析 `snaplen` 和 `capacity`。
2. 校验参数范围。
3. 创建 `PacketFsInner`。
4. 创建 root、packets、stats 三个 inode。
5. 设置 `mounted=true`。
6. 返回 root inode。

`PacketFs::umount()`：

1. 调用 `begin_active_umount()`。
2. 如果仍有 `/packets` reader，返回 `EBUSY`。
3. 将活跃实例置为 unmounted，清空 ring，唤醒等待队列。
4. 清除 active instance，使后续收包返回 `DroppedInactive`。

## 错误处理

- 参数无法解析：`EINVAL`。
- 参数越界：`EINVAL`。
- 内存分配失败：`ENOMEM`。
- 已有活跃挂载实例时，如果实现单实例限制，返回 `EBUSY`。
