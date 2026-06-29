# sync 目录导引

## 额外说明

大部分参考 rCore，`wait_queue` 按本项目阻塞读模型适配。

## 目录职责

`sync/` 提供内核同步原语。当前内核 crate 可以继续使用 `std` 的集合、`Arc`、`OnceLock` 等能力，但共享状态锁统一走本目录的自写实现，不直接使用 `std::sync::Mutex`、`RwLock` 或 `Condvar`。

packetfs 使用它保护：

- packet ring。
- stats。
- file 内部状态。
- reader_active。
- wait queue。
