# src/task/id.rs

## 额外说明

照搬 rCore。

## 文件职责

分配 pid、tid、kernel stack 等任务标识资源。

## packetfs 关系

只用于运行用户 demo 进程，不参与 packetfs 文件系统逻辑。
