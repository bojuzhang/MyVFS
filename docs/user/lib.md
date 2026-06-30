# user/src/lib.rs

## 额外说明

当前 host model 下使用普通 `std` crate，不再提供裸机 `_start` 或 panic handler。

## 文件职责

提供 `print`/`println`、`exit` 调用和基础库导出。

## packetfs 关系

`packetdump` 和 `packetstat` 作为普通 host 用户程序运行，依赖该文件提供 syscall wrapper 和打印宏。
