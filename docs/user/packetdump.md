# user/src/bin/packetdump.rs

## 文件职责

读取 `/mnt/packetfs/packets`，把 PCAP 字节流通过串口以 hex 形式导出给 host。

## 核心流程

```text
mount("packetfs", "/mnt/packetfs", "snaplen=2048,capacity=256")
fd = open("/mnt/packetfs/packets", O_RDONLY)
print "PCAP_BEGIN"
loop:
    n = read(fd, buf)
    if n > 0:
        print hex(buf[..n])
    if enough bytes for demo:
        break
print "PCAP_END"
close(fd)
```

## 需要维护的信息

- 读取 buffer。
- 已导出字节数。
- 最大导出字节数或最大 packet 数，防止 demo 无限运行。

## 错误处理

- mount 失败：打印错误并退出。
- open 失败：打印错误并退出。
- read 返回负值：打印错误并退出。

## 与 packetfs 的关系

这是最终展示的主程序，证明用户态通过 VFS 读取 PCAP。
