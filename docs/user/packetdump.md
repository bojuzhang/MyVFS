# user/src/bin/packetdump.rs

## 文件职责

演示 `packetfs` 的 VFS 行为，并读取 `/mnt/packetfs/packets`，把 PCAP 字节流通过串口以 hex 形式导出给 host。

## 核心流程

```text
stat("/mnt/packetfs")
mount("packetfs", "/mnt/packetfs", "snaplen=2048,capacity=256")
stat("/mnt/packetfs")
getdents("/mnt/packetfs") -> ".", "..", "packets", "stats"
open("/mnt/packetfs/packets", O_WRONLY) -> EACCES
inject 3 demo Ethernet frames
fd = open("/mnt/packetfs/packets", O_RDONLY)
open("/mnt/packetfs/packets", O_RDONLY) -> EBUSY
write(fd, "packetfs-write-attempt") -> EACCES
loop until 3 records:
    n = read(fd, buf)
    print READ_CHUNK with phase, record index, hex and ascii preview
close(fd)
decode records for human-readable payload summary
print "PCAP_BEGIN"
print exported PCAP hex
print "PCAP_END"
read("/mnt/packetfs/stats") and print content
umount("/mnt/packetfs")
stat("/mnt/packetfs")
```

## 需要维护的信息

- 读取 buffer。
- 已导出字节数。
- 最大导出字节数或最大 packet 数，防止 demo 无限运行。
- PCAP 流解析状态，用于把 global header、record header、record payload 分多次 read 展示。
- 演示帧的 Ethernet header、payload 和提交结果。

## 错误处理

- mount 失败：打印错误并退出。
- open 失败：打印错误并退出。
- read 返回负值：打印错误并退出。
- 目录项解析、PCAP 解析、close、umount 失败：打印错误并退出。

## 与 packetfs 的关系

这是最终展示的主程序，证明用户态通过普通 VFS syscall 完成挂载、目录读取、文件 metadata 查询、只读访问控制、单读者管理、PCAP 流读取和 stats 文件读取。
