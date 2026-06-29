# src/fs/stat.rs

## 文件职责

定义文件类型、文件元数据、用户态 stat ABI 和目录项结构。

## 需要定义的 enum

```rust
pub enum FileType {
    Directory,
    Regular,
    Pipe,
    CharDevice,
}
```

## 需要定义的 struct

```rust
pub struct Metadata {
    pub inode_id: u64,
    pub file_type: FileType,
    pub mode: u16,
    pub size: usize,
    pub nlink: u16,
}
```

```rust
pub struct Stat {
    pub inode_id: u64,
    pub mode: u16,
    pub size: usize,
    pub file_type: u16,
}
```

```rust
pub struct DirEntry {
    pub name: String,
    pub inode_id: u64,
    pub file_type: FileType,
}
```

## 权限约定

- 目录：`0555`
- 普通只读文件：`0444`

## 核心流程

- `Inode::metadata()` 返回 `Metadata`。
- `sys_stat()` 将 `Metadata` 转成用户 ABI `Stat`。
- `Inode::readdir()` 返回 `Vec<DirEntry>`，`sys_getdents()` 拷贝给用户态。