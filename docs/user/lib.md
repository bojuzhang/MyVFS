# user/src/lib.rs

## 额外说明

照搬 rCore。

## 文件职责

提供用户程序入口、panic 处理、exit 调用和基础库导出。

## packetfs 关系

`packetdump` 和 `packetstat` 作为普通用户程序运行，依赖该文件提供入口包装。
