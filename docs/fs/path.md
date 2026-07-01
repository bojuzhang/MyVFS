# src/fs/path.rs

## 文件职责

负责把用户传入的路径字符串解析为 VFS inode。

## 需要定义的 struct

```rust
pub struct Path {
    pub raw: String,
    pub components: Vec<String>,
    pub is_absolute: bool,
}
```

```rust
pub struct PathResolver<'a> {
    pub root: DynInode,
    pub cwd: DynInode,
    pub mount_table: &'a MountTable,
}
```

## 需要实现的接口

```rust
impl Path {
    pub fn parse(raw: &str) -> FsResult<Self>;
}
```

```rust
impl<'a> PathResolver<'a> {
    pub fn new(root: DynInode, cwd: DynInode, mount_table: &'a MountTable) -> Self;
    pub fn resolve(&self, path: &str) -> FsResult<DynInode>;
    pub fn resolve_parent(&self, path: &str) -> FsResult<(DynInode, String)>;
}
```

## 维护的信息

- 原始路径。
- 规范化后的组件。
- 是否绝对路径。
- 当前工作目录快照，由 `TaskControlBlock` 提供，`PathResolver` 不拥有进程 cwd 状态。
- 根目录。
- 全局挂载表引用，解析器只借用它，不持有 `Arc<MountTable>`。

## 核心流程

`resolve("")`：

1. 检查路径非空。
2. 判断是否绝对路径。
3. 合并重复 `/`。
4. 忽略 `.`。
5. 处理 `..`：能够通过已遍历祖先栈回退时直接回退；相对路径开头等没有栈可退的情况调用当前 inode 的 `lookup("..")`，在根目录上保持不动。
6. 绝对路径从 root 开始，相对路径从当前任务的 cwd 快照开始，逐级调用 `lookup()`。
7. 每进入一个 inode 后调用 `MountTable::follow_mount()`。
8. 返回最终 inode。

`resolve_parent("")`：

1. 解析到上一级。
2. 返回父 inode 和最后一级名字。

## 错误处理

- 空路径：`EINVAL`。
- 中间路径不存在：`ENOENT`。
- 中间节点不是目录：`ENOTDIR`。
- 最后一级不存在：`ENOENT`。
