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
    pub parent: Weak<RamInode>,
    pub next_inode_id: Arc<AtomicU64>,
    pub inner: Arc<Mutex<RamInodeInner>>,
    self_ref: Weak<RamInode>,
}
```

```rust
pub enum RamInodeInner {
    Directory(BTreeMap<String, DynInode>),
    File(Vec<u8>),
}
```

这里的 `Mutex` 是 `crate::sync::Mutex`，不是 `std::sync::Mutex`。
`parent` 和 `self_ref` 使用 `std::sync::Weak`，避免目录 children 持有子节点 `Arc` 后再由子节点强持有父节点形成引用环。

```rust
pub struct RamFile {
    pub inode: Arc<RamInode>,
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
- 每个 inode 持有父节点弱引用；根目录的父节点指向根目录自身。
- 每个 inode 持有自身弱引用，供 `mkdir/create_child` 在只有 `&self` 的 trait 方法里给新子节点设置父节点。
- 文件大小。
- 打开文件的 offset 由 `FileHandle` 维护，`RamFile` 不保存 offset。

## 核心流程

目录 `lookup()`：

1. 检查当前 inode 是目录。
2. 如果名字是 `..`，升级 `parent` 弱引用并返回父 inode；根目录返回自身。
3. 否则在 `children` 中查找名字。
4. 找到返回 inode，否则 `ENOENT`。

目录 `readdir()`：

1. 返回 `.`, `..` 和 children。
2. `.` 使用当前 inode id，`..` 使用父 inode id；根目录两者相同。

目录 `mkdir(name)`：

1. 检查名字合法且当前 inode 是目录。
2. 使用共享 inode id 分配器创建目录 inode。
3. 将新 inode 的 `parent` 指向当前目录。
4. 插入 children；名字已存在返回 `EBUSY`。

文件 `read_at()`：

1. 从 Vec 中按 offset 拷贝。
2. 超出长度返回 0。

`RamFile::read(offset, buf)`：

1. 使用调用方传入的 `FileHandle` offset。
2. 调用 `RamInode::read_at(offset, tmp)` 读取 inode 内容。
3. 将临时缓冲区拷贝到 `UserBuffer`，返回本次实际拷贝字节数；offset 推进由 `FileHandle` 完成。

文件 `write_at()`：

1. 如果 ramfs 支持写，自动扩容 Vec。
2. 如果暂不支持写，返回 `EROFS`。
