# sync 目录导引

## 额外说明

大部分照搬 rCore，`wait_queue` 可新增或改写。

## 目录职责

`sync/` 提供内核同步原语。packetfs 使用它保护：

- packet ring。
- stats。
- file 内部状态。
- reader_active。
- wait queue。
