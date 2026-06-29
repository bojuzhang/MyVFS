# packetfs 目录导引

## 目录职责

`src/fs/packetfs/` 是本项目核心。它实现一个可挂载伪文件系统，挂载后目录结构固定为：

```text
/
  packets
  stats
```

## 文件语义

- `/packets`：只读、阻塞、单读者、classic PCAP 流。
- `/stats`：只读、非阻塞、文本统计快照。

## 下层入口

virtio-net 收到 Ethernet frame 后只调用：

```rust
packetfs::api::submit_rx_frame(frame, meta)
```

virtio-net 不解析 IP，不访问用户缓冲区，不写 PCAP。

## 固定不变量

- `submit_rx_frame()` 不睡眠。
- ring 有界。
- ring 满时丢新包。
- `/packets` 只允许一个 reader。
- `/packets` 每次 open 先输出 PCAP global header。
- `/packets` 不支持 seek。
- `packetfs` 不支持写、创建、删除、truncate。