# fs 目录导引

## 目录职责

`src/fs/` 提供教学内核的 VFS 基础层：

- 定义文件系统、inode、file 抽象。
- 维护路径解析和挂载表。
- 管理进程 fd table。
- 提供根目录 ramfs。
- 注册并挂载 `packetfs`。
- 给 syscall 层提供 `open_path/mount_fs/umount_fs` 等入口。

