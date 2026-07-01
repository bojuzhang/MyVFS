# src/fs/mod.rs

## 文件职责

`mod.rs` 负责导出公共类型、初始化全局文件系统状态，并向 syscall 层提供稳定入口。

## 需要定义的模块导出

```rust
pub mod error;
pub mod stat;
pub mod vfs;
pub mod path;
pub mod mount;
pub mod fd;
pub mod ramfs;
pub mod packetfs;
pub mod stdio;
```

公开 re-export：

```rust
pub use error::{FsError, FsResult};
pub use stat::{FileType, Metadata, Stat, DirEntry};
pub use vfs::{FileSystem, Inode, File, DynInode, DynFile};
pub use fd::{FdTable, FileHandle, OpenFlags};
```

## 需要维护的信息

- `ROOT_FS`：根 ramfs。
- `MOUNT_TABLE`：全局挂载表。
- `FS_REGISTRY`：文件系统类型注册表。
- `resolver()`：从 `current_task()` 取得当前任务 cwd，并把 root、cwd 和挂载表引用传给 `PathResolver`。

这些全局状态由 `crate::sync::Mutex` 保护。当前内核 crate 允许继续使用 `std::collections`、`Arc`、`OnceLock` 等非锁能力，但不要为共享状态新增 `std::sync` 锁。

## 需要实现的接口

- `pub fn init() -> FsResult<()>`
- `pub fn register_filesystem(fs: Arc<dyn FileSystem>) -> FsResult<()>`
- `pub fn open_path(path: &str, flags: OpenFlags) -> FsResult<DynFile>`
- `pub fn mount_fs(fs_name: &str, target: &str, options: &str) -> FsResult<()>`
- `pub fn umount_fs(target: &str) -> FsResult<()>`
- `pub fn stat_path(path: &str) -> FsResult<Metadata>`
- `pub fn read_dir_path(path: &str) -> FsResult<Vec<DirEntry>>`

## 核心流程

`fs::init()`：

1. 创建 `RamFs` 作为 `/`。
2. 在 ramfs 中创建 `/mnt`。
3. 在 ramfs 中创建 `/mnt/packetfs` 作为挂载点。
4. 初始化 `MountTable`，根节点指向 ramfs root。
5. 注册 `packetfs` 文件系统类型。
6. 初始化 stdin/stdout 文件对象所需状态。

## 错误处理

- 重复初始化：`EBUSY`。
- 注册重复文件系统：`EBUSY`。
- 找不到文件系统类型：`ENODEV`。
- 当前任务不存在或 cwd 无法取得：`EIO`。
- 路径解析错误透传自 `path.rs`。
