# src/fs/mount.rs

## 文件职责

维护全局挂载表

## 需要定义的 struct

```rust
pub struct MountEntry {
    pub target_path: String,
    pub mountpoint_inode: DynInode,
    pub root_inode: DynInode,
    pub fs_name: &'static str,
}
```

```rust
pub struct MountTable {
    pub root: DynInode,
    pub entries: Vec<MountEntry>,
}
```

## 需要实现的接口

```rust
impl MountTable {
    pub fn new(root: DynInode) -> Self;
    pub fn mount(&self, fs: DynFileSystem, target: &str, options: &str) -> FsResult<()>;
    pub fn umount(&self, target: &str) -> FsResult<()>;
    pub fn follow_mount(&self, inode: &DynInode) -> DynInode;
    pub fn is_mountpoint(&self, inode: &DynInode) -> bool;
}
```

## 维护的信息

- 全局根 inode。
- 每个挂载点 inode。
- 被挂载文件系统的根 inode。
- 文件系统类型名字。
- 目标路径，便于 `umount` 和调试。

## 核心流程

`mount("packetfs", "/mnt/packetfs", options)`：

1. 查找 `packetfs` 类型。
2. resolve target。
3. 确认 target 是目录。
4. 确认 target 未被挂载。
5. 调用 `PacketFs::mount(options)`。
6. 插入 `MountEntry`。

路径解析时：

1. `PathResolver` 每得到一个 inode。
2. 调用 `follow_mount(inode)`。
3. 如果 inode 是挂载点，返回对应 `root_inode`。
4. 否则返回原 inode。

## 限制

- 单全局 mount table。
- 不做 mount namespace。
- 不做 bind mount。
- 不做递归挂载。
- 不做复杂 busy 检查；但 `packetfs` 有 reader 时 `umount` 应返回 `EBUSY`。