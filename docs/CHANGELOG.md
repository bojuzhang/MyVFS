# 文档更新记录

## Unreleased

- 将 VFS 卸载流程改为通过 `FileSystem::umount()` 动态分发，`MountTable` 不再按 `packetfs` 名字特判卸载逻辑。
- 明确内核 crate 当前允许使用 `std`，但内核共享状态锁必须使用 `kernel/src/sync/` 的自写实现。
- 同步 `sync` 文档：`Mutex<T>` 现在是 `SpinMutex<T>` 的项目内实现，基于 `AtomicBool` 和 `UnsafeCell`，不再包装 `std::sync::Mutex`。
- 同步 `WaitQueue` 文档：阻塞读使用 `prepare_wait()` 与 `sleep_current_with_guard()` 保留检查条件到入睡之间的 gate，收包后通过 `wake_one()` 唤醒。
- 同步 `packetfs`、`fs`、`task` 相关文档中的锁来源和等待队列描述，避免继续暗示可使用标准库锁。
- 调整演示与运行文档口径：当前默认运行模型为保留部分 `std` 的 host model，QEMU/no_std RISC-V 真实内核作为后续迁移目标。
- 将默认演示脚本切到 host model：用户程序以普通 `std` 可执行程序运行，非 RISC-V syscall wrapper 直接调用内核 host syscall，QEMU 路径保留为显式 legacy 模式。
