# src/fs/mount.rs

## 文件职责

维护全局挂载表

## 需要定义的 struct

```rust
pub struct MountEntry {
    pub target_path: String,
    pub mountpoint_inode: DynInode,
    pub root_inode: DynInode,
    pub fs: DynFileSystem,
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
- 被挂载文件系统对象，用于在 `umount` 时通过 `FileSystem::umount()` 做文件系统私有清理。
- 目标路径，便于 `umount` 和调试。

## 核心流程

`mount("packetfs", "/mnt/packetfs", options)`：

1. 查找 `packetfs` 类型。
2. resolve target。
3. 确认 target 是目录。
4. 确认 target 未被挂载。
5. 调用 `FileSystem::mount(options)`。
6. 插入 `MountEntry`。

`/mnt/packetfs` 是 packetfs API 暴露的默认挂载点；`MountTable` 只接收调用方传入的 target，不维护该默认路径。

`umount("/mnt/packetfs")`：

1. 按 target 找到 `MountEntry`。
2. 调用 `entry.fs.umount()`。
3. 文件系统私有卸载成功后从挂载表移除该项。

`MountTable` 不根据文件系统名字特判卸载逻辑；例如 `packetfs` 的 reader busy 检查和 active instance 清理由 `PacketFs::umount()` 自己完成。

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
