# src/fs/fd.rs

## 文件职责

管理每个进程的文件描述符表，把整数 fd 映射到打开后的 `File` 对象和打开状态。

## 需要定义的 flags

```rust
bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 2;
        const TRUNC = 1 << 3;
        const DIRECTORY = 1 << 4;
    }
}
```

需要实现：

```rust
impl OpenFlags {
    pub fn readable(&self) -> bool;
    pub fn writable(&self) -> bool;
}
```

## 需要定义的 struct

```rust
pub struct FileHandle {
    pub file: DynFile,
    pub flags: OpenFlags,
    pub offset: usize,
    pub debug_path: String,
}
```

```rust
pub struct FdTable {
    pub entries: Vec<Option<FileHandle>>,
}
```

## 需要实现的接口

```rust
impl FdTable {
    pub fn new() -> Self;
    pub fn alloc(&mut self, handle: FileHandle) -> FsResult<usize>;
    pub fn get(&self, fd: usize) -> FsResult<&FileHandle>;
    pub fn get_mut(&mut self, fd: usize) -> FsResult<&mut FileHandle>;
    pub fn close(&mut self, fd: usize) -> FsResult<()>;
    pub fn dup(&mut self, fd: usize) -> FsResult<usize>;
    pub fn fork_clone(&self) -> Self;
}
```

## 维护的信息

- fd 是否被占用。
- fd 对应 file 对象。
- 打开权限。
- 当前 offset。
- 调试路径。

## 核心流程

`alloc()`：

1. 从 fd 0 开始找空位。
2. 插入 `FileHandle`。
3. 返回 fd。

`close()`：

1. 找到 fd。
2. 调用 `file.close()`。
3. 将表项置空。

`sys_read()` 使用 fd table：

1. 检查 fd 存在。
2. 检查 handle flags 可读。
3. 取出 `FileHandle.offset` 作为本次读取位置。
4. 调用 `handle.file.read(offset, user_buffer)`。
5. 按实际读取字节数推进 `FileHandle.offset`。
