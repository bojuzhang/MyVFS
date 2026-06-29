# src/task/switch.rs / switch.S

## 额外说明

照搬 rCore。

## 文件职责

实现底层上下文切换汇编或封装。

## packetfs 关系

`/packets` 阻塞读需要调度切换，但 packetfs 不修改 switch 实现。