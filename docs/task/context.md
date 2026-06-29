# src/task/context.rs

## 额外说明

照搬 rCore。

## 文件职责

保存任务切换所需寄存器上下文。

## packetfs 关系

无直接关系。packetfs 阻塞读最终会触发任务切换，但不需要理解或修改寄存器保存格式。
