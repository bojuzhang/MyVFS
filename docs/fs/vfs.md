## 文件职责

定义 VFS 层的核心 trait。具体文件系统只要实现这些 trait，就能被路径解析、mount、syscall 和 fd table 使用。

## 需要定义的类型别名

```rust
pub type DynInode = Arc<dyn Inode + Send + Sync>;
pub type DynFile = Arc<dyn File + Send + Sync>;
pub type DynFileSystem = Arc<dyn FileSystem + Send + Sync>;
```

## trait FileSystem

```rust
pub trait FileSystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn mount(&self, options: &str) -> FsResult<DynInode>;
    fn root_inode(&self) -> DynInode;
}
```

解释：

- `name()` 用于注册表查找。
- `mount()` 创建一次挂载实例，返回该实例根 inode。
- `root_inode()` 用于已有实例返回根节点。

## trait Inode

```rust
pub trait Inode: Send + Sync {
    fn metadata(&self) -> FsResult<Metadata>;
    fn lookup(&self, name: &str) -> FsResult<DynInode>;
    fn readdir(&self) -> FsResult<Vec<DirEntry>>;
    fn open(&self, flags: OpenFlags) -> FsResult<DynFile>;

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> FsResult<usize>;
    fn write_at(&self, offset: usize, buf: &[u8]) -> FsResult<usize>;
}
```

解释：

- 目录 inode 实现 `lookup/readdir`。
- 普通文件 inode 对 `lookup/readdir` 返回 `ENOTDIR`。
- `read_at/write_at` 主要给 ramfs；`packetfs` 的 `/packets` 以 `File::read` 维护流状态。

## trait File

```rust
pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> FsResult<usize>;
    fn write(&self, buf: UserBuffer) -> FsResult<usize>;
    fn stat(&self) -> FsResult<Metadata>;
    fn close(&self) -> FsResult<()>;
    fn seek(&self, pos: SeekFrom) -> FsResult<usize>;
}
```

## 需要定义的 enum

```rust
pub enum SeekFrom {
    Start(usize),
    Current(isize),
    End(isize),
}