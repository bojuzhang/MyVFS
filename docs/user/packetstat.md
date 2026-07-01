# user/src/bin/packetstat.rs

## 文件职责

读取 packetfs 默认 `/stats` 路径并打印统计文本。

## 核心流程

```text
fd = open("/mnt/packetfs/stats", O_RDONLY)
loop read until EOF
print content
close(fd)
```

## 需要验证的字段

- `captured_packets`
- `captured_bytes`
- `read_packets`
- `read_bytes`
- `queued_packets`
- `dropped_full`
- `dropped_inactive`
- `truncated_packets`
- `reader_active`

## 与 packetfs 的关系

`packetstat` 展示文件系统内部状态，证明 packetfs 不只是一个单文件 read wrapper，而是在维护自己的状态。
默认挂载点和 stats 路径引用 packetfs 导出的常量，避免用户程序重复维护 `/mnt/packetfs`。
