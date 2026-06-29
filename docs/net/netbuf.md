# src/net/netbuf.rs

## 文件职责

定义或说明网络帧在内核中的最小表达。packetfs 需要的是 Ethernet frame 字节切片。

## 可选 struct

```rust
pub struct NetBuf {
    pub data: Vec<u8>,
    pub len: usize,
}
```

如果 virtio-net RX 路径已经能提供 `&[u8]`，则不必单独定义 `NetBuf`。

## frame 要求

提交给 packetfs 的 frame 必须是：

- Ethernet frame。
- 不包含 virtio-net header。
- 不包含 Ethernet FCS。
- 不要求 IP/TCP/UDP 有效。

## 与 packetfs 的关系

最终调用：

```rust
packetfs::submit_rx_frame(frame, RxMeta { timestamp_us, iface_id })
```