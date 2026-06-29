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
pub struct PathResolver {
    pub root: DynInode,
    pub cwd: DynInode,
    pub mount_table: Arc<MountTable>,
}
```

## 需要实现的接口

```rust
impl Path {
    pub fn parse(raw: &str) -> FsResult<Self>;
}
```

```rust
impl PathResolver {
    pub fn resolve(&self, path: &str) -> FsResult<DynInode>;
    pub fn resolve_parent(&self, path: &str) -> FsResult<(DynInode, String)>;
}
```

## 维护的信息

- 原始路径。
- 规范化后的组件。
- 是否绝对路径。
- 当前工作目录。
- 根目录。
- 挂载表引用。

## 核心流程

`resolve("")`：

1. 检查路径非空。
2. 判断是否绝对路径。
3. 合并重复 `/`。
4. 忽略 `.`。
5. 处理 `..`，在根目录上保持不动。
6. 从 root 或 cwd 开始逐级调用 `lookup()`。
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