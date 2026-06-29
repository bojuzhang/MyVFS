# src/sync/up.rs

## 额外说明

参考 rCore。

## 文件职责

提供单核教学内核中的内部可变性封装，例如 `UPSafeCell<T>`。

当前实现内部使用 `crate::sync::Mutex<T>`，因此仍满足“锁由项目自写”的约束。

## packetfs 使用方式

可用于保护全局对象：

- `FS_REGISTRY`
- `MOUNT_TABLE`
- `ACTIVE_PACKETFS`

## 注意事项

如果内核开启多核或中断并发，单纯 `UPSafeCell` 不够，packet ring 应使用能关中断或自旋的锁。
