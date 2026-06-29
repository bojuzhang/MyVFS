# src/net/virtio_net.rs

## 额外说明

可照搬/最小改写 rCore。

virtio-mmio、virtqueue 初始化可以参考 rCore 或 `virtio-drivers`。本项目只要求 RX 路径能拿到 Ethernet frame 并调用 packetfs API。

## 文件职责

初始化 QEMU virtio-net 设备，轮询或处理中断得到 RX buffer，并把收到的 Ethernet frame 交给 packetfs。

## 需要说明的状态

- virtio-net 设备对象。
- RX queue。
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
