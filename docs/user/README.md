# user 目录导引

## 目录职责

用户程序证明 `packetfs` 已接入 VFS。它们只调用标准文件 syscall，不调用内核私有接口。
默认路径可以引用 packetfs 导出的公共常量，但实际访问仍必须经由 `mount/open/read/stat/getdents/umount` 等 syscall wrapper。
