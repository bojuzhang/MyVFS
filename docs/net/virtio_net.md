# src/net/virtio_net.rs

## 文件职责

`virtio_net` 提供 packetfs 所需的网卡 RX 输入层。它在 QEMU `virt` 平台上初始化 virtio-net 设备，接收原始 Ethernet frame，并把 frame 通过 packetfs 的稳定入口提交给文件系统

当前实现是 host model 下的 RX frame 注入层，提供队列、时间戳和 `packetfs::submit_rx_frame()` 调用路径。它不表示已经完成 QEMU virtio-mmio 驱动。

后续迁移到 QEMU/no_std 真实内核时，virtio-mmio、virtqueue 初始化可以参考 rCore 或 `virtio-drivers`。packetfs 只要求 RX 路径能拿到 Ethernet frame 并调用 packetfs API。

## 需要说明的状态

- virtio-net 设备对象。
- RX queue，由 `crate::sync::Mutex` 保护。
- RX buffer。
- 可选 iface id。

## RX 核心流程

```text
virtio_net::poll_rx()
 -> 从 virtqueue 取 used buffer
 -> 解析并跳过 virtio_net_hdr
 -> 得到 Ethernet frame &[u8]
 -> 生成 RxMeta
 -> packetfs::api::submit_rx_frame(frame, meta)
 -> 归还/重新投递 RX buffer
```

## 不做内容

- 不实现 TX。
- 不实现 IP 栈。
- 不实现 checksum offload。
- 不实现多队列。
- 不直接写用户 buffer。
- 不生成 PCAP。
