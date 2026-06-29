# src/fs/error.rs

## 文件职责

统一定义 VFS、ramfs、packetfs、syscall fs 层使用的错误类型，并提供到 syscall 返回值的转换。

## 需要定义的 enum

```rust
pub enum FsError {
    Enoent,
    Enotdir,
    Eisdir,
    Einval,
    Ebusy,
    Erofs,
    Eio,
    Enomem,
    Enodev,
    Espipe,
    Eacces,
    Eagain,
}
```

## 类型别名

```rust
pub type FsResult<T> = Result<T, FsError>;
```

## 需要实现的 impl

```rust
impl FsError {
    pub fn as_isize(&self) -> isize;
    pub fn name(&self) -> &'static str;
}
```

`as_isize()` 返回负 errno，供 syscall 层直接返回给用户态。

## 错误使用场景

| 错误 | 场景 |
|---|---|
| `ENOENT` | 路径或目录项不存在。 |
| `ENOTDIR` | 对普通文件继续 lookup 或 readdir。 |
| `EISDIR` | 把目录当普通文件读写。 |
| `EINVAL` | mount 参数非法、空路径、非法 flag。 |
| `EBUSY` | 重复 mount、单读者已存在、umount 时仍有 reader。 |
| `EROFS` | 写只读文件系统或写 `/packets`、`/stats`。 |
| `EIO` | 卸载中、底层状态失效。 |
| `ENOMEM` | 分配 packet record、inode、file 失败。 |
| `ENODEV` | 文件系统未注册或 packetfs 未挂载。 |
| `ESPIPE` | 对 `/packets` seek。 |
| `EACCES` | 以不允许的权限打开文件。 |
| `EAGAIN` | 非阻塞读空队列；本项目默认阻塞，可仅作为扩展保留。 |