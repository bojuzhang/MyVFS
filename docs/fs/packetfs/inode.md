# src/fs/packetfs/inode.rs

## 分类

新增实现。

## 文件职责

定义 `packetfs` 的 inode 层，负责目录树、lookup、readdir、metadata 和 open。

## 需要定义的 enum

```rust
pub enum PacketNodeKind {
    Root,
    Packets,
    Stats,
}
```

## 需要定义的 struct

```rust
pub struct PacketInode {
    pub fs: Arc<PacketFsInner>,
    pub kind: PacketNodeKind,
    pub inode_id: u64,
}
```

固定 inode id：

```text
1: root
2: packets
3: stats
```

## 需要实现的 impl

```rust
impl PacketInode {
    pub fn new_root(fs: Arc<PacketFsInner>) -> Self;
    pub fn new_packets(fs: Arc<PacketFsInner>) -> Self;
    pub fn new_stats(fs: Arc<PacketFsInner>) -> Self;
}
```

```rust
impl Inode for PacketInode {
    fn metadata(&self) -> FsResult<Metadata>;
    fn lookup(&self, name: &str) -> FsResult<DynInode>;
    fn readdir(&self) -> FsResult<Vec<DirEntry>>;
    fn open(&self, flags: OpenFlags) -> FsResult<DynFile>;
}
```

## metadata 规则

| 节点 | 类型 | 权限 | size |
|---|---|---|---|
| root | Directory | `0555` | 0 |
| packets | Regular | `0444` | 0 |
| stats | Regular | `0444` | 动态或 0 |

## lookup 规则

root:

```text
lookup(".")       -> root
lookup("..")      -> root
lookup("packets") -> packets inode
lookup("stats")   -> stats inode
lookup(other)     -> ENOENT
```

packets/stats:

```text
lookup(any) -> ENOTDIR
```

## readdir 规则

root 返回：

```text
.
..
packets
stats
```

packets/stats 返回 `ENOTDIR`。

## open 规则

root：

- 允许目录打开，用于 `getdents`。
- 写打开返回 `EISDIR` 或 `EACCES`。

packets：

- 只允许只读。
- 如果 `reader_active == true`，返回 `EBUSY`。
- 成功后创建 `PacketCaptureFile`。

stats：

- 只允许只读。
- 允许多个 reader。
- 成功后创建 `StatsFile`。
