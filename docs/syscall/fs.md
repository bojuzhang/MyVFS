# src/syscall/fs.rs

## 额外说明

照搬/改写 rCore。注意需要接入本项目的 VFS。

## 文件职责

实现文件系统相关 syscall，是用户态访问 `packetfs` 的唯一入口。

## 需要实现的 syscall

```rust
pub fn sys_mount(fs_name: *const u8, target: *const u8, options: *const u8) -> isize;
pub fn sys_umount(target: *const u8) -> isize;
pub fn sys_open(path: *const u8, flags: u32) -> isize;
pub fn sys_close(fd: usize) -> isize;
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize;
pub fn sys_stat(path: *const u8, stat_ptr: *mut Stat) -> isize;
pub fn sys_getdents(fd: usize, buf: *mut u8, len: usize) -> isize;
```

## 需要使用的辅助

- `translated_str()`：复制用户态字符串。
- `translated_byte_buffer()`：翻译用户缓冲区。
- 当前任务的 `FdTable`。
- `fs::mount_fs/open_path/stat_path/read_dir_path`。

## sys_mount 流程

1. 从用户态复制 `fs_name`、`target`、`options`。
2. 调用 `fs::mount_fs(fs_name, target, options)`。
3. 成功返回 0，失败返回负 errno。

## sys_open 流程

1. 复制 path。
2. 解析 flags。
3. 调用 `fs::open_path(path, flags)`。
4. 创建 `FileHandle`。
5. 插入当前任务 fd table。
6. 返回 fd。

## sys_read 流程

1. 根据 fd 找到 `FileHandle`。
2. 检查可读权限。
3. 翻译用户 buffer。
4. 调用 `file.read(user_buffer)`。
5. 返回读取字节数。

注意：`sys_read` 不知道 `PacketCaptureFile`，也不直接访问 packet queue。

## sys_getdents 流程

1. 根据 fd 找到目录 file。
2. 读取目录项。
3. 按用户 ABI 写入用户 buffer。
4. 返回写入字节数。

## 错误处理

- 用户字符串非法：`EINVAL` 或 `EFAULT`，若没有 `EFAULT` 可用 `EINVAL`。
- fd 不存在：`EINVAL`。
- 权限不符：`EACCES`。
- VFS 错误透传。