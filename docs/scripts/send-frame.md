# scripts/send-frame.py

## 文件职责

在 host 侧构造一帧 Ethernet frame 并写入 `tap0`，触发 guest virtio-net RX。

## 帧结构

```text
dst_mac: 6 bytes
src_mac: 6 bytes
ethertype: 2 bytes
payload: N bytes
```

建议使用自定义 ethertype，例如 `0x88b5`，payload 写入可识别字符串。

## 脚本流程

1. 打开 TAP 设备。
2. 构造 Ethernet frame。
3. 写入 frame。
4. 打印发送长度。

## 错误处理

- 没有权限打开 TAP：提示 sudo 或权限设置。
- tap0 不存在：提示先运行 setup-tap。

## 与 packetfs 的关系

该脚本制造 packetfs 的输入数据，但不是内核核心实现。
