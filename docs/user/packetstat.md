# user/src/bin/packetstat.rs

## 文件职责

读取 `/mnt/packetfs/stats` 并打印统计文本。

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