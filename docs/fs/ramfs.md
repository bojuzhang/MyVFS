# src/fs/ramfs.rs

## 分类

新增实现。

## 文件职责

提供一个内存根文件系统，让 `packetfs` 有真实挂载点。

## 需要定义的 struct

```rust
pub struct RamFs {
    pub root: Arc<RamInode>,
    pub next_inode_id: Arc<AtomicU64>,
}
```

```rust
pub struct RamInode {
    pub id: u64,
    pub name: String,
    pub kind: FileType,
    pub next_inode_id: Arc<AtomicU64>,
    pub inner: Arc<Mutex<RamInodeInner>>,
}
```

```rust
pub enum RamInodeInner {
    Directory(BTreeMap<String, DynInode>),
    File(Vec<u8>),
}
```

这里的 `Mutex` 是 `crate::sync::Mutex`，不是 `std::sync::Mutex`。

```rust
pub struct RamFile {
    pub inode: Arc<RamInode>,
    pub offset: Mutex<usize>,
    pub flags: OpenFlags,
}
```

## 需要实现的 impl

- `impl FileSystem for RamFs`
- `impl Inode for RamInode`
- `impl File for RamFile`

## 维护的信息

- 内存目录树。
- 文件内容 Vec。
- inode id。
- 每个目录 inode 持有共享 inode id 分配器，使 VFS `mkdir` 可以从父 inode 创建子目录。
- 文件大小。
- 每个打开文件的 offset。

## 核心流程

目录 `lookup()`：

1. 检查当前 inode 是目录。
2. 在 `children` 中查找名字。
3. 找到返回 inode，否则 `ENOENT`。

目录 `readdir()`：

1. 返回 `.`, `..` 和 children。

目录 `mkdir(name)`：

1. 检查名字合法且当前 inode 是目录。
2. 使用共享 inode id 分配器创建目录 inode。
3. 插入 children；名字已存在返回 `EBUSY`。

文件 `read_at()`：

1. 从 Vec 中按 offset 拷贝。
2. 超出长度返回 0。

文件 `write_at()`：

1. 如果 ramfs 支持写，自动扩容 Vec。
2. 如果暂不支持写，返回 `EROFS`。
