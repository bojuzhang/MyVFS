# syscall 目录导引

## 目录职责

`src/syscall/` 是用户态进入内核 VFS 的入口。`packetfs` 必须通过普通文件 syscall 展示，而不是新增专用抓包 syscall。

## 固定原则

- syscall 层不直接调用 virtio-net。
- syscall 层不直接操作 packet queue。
- `sys_read` 只通过 fd 找到 `File` 和当前 offset，再调用 `File::read(offset, ...)`。
- `sys_open` 只通过路径解析找到 inode，再调用 `Inode::open()`。
