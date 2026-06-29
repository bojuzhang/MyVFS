# net 目录导引

## 说明

demo 支撑为主，virtio-net 可照搬/最小改写 rCore。

## 目录职责

`src/net/` 只负责给 `packetfs` 提供原始 Ethernet frame 来源。它不是完整网络协议栈。

## 固定原则

- 不实现 TCP/IP。
- 不实现 ARP。
- 不实现 ping。
- 不实现 TX 发包。
- 不提供 socket API。
- virtio-net 收到 RX frame 后调用 `packetfs::submit_rx_frame()`。
