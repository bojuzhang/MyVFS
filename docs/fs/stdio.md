# src/fs/stdio.rs

## 文件职责

把标准输入输出适配为 `File`，供用户程序打印 demo 信息和串口输出 PCAP hex。

## 需要定义的 struct

```rust
pub struct Stdin;
pub struct Stdout;
```

## 需要实现的 impl

`impl File for Stdin`：

- `readable() -> true`
- `writable() -> false`
- `read()` 从 console 读取。
- `write()` 返回 `EBADF` 或 `EACCES`。

`impl File for Stdout`：

- `readable() -> false`
- `writable() -> true`
- `write()` 输出到 console/serial。
- `read()` 返回错误。