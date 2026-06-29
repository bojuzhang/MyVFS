# scripts/setup-tap.sh

## 文件职责

创建 host 侧 TAP 设备，供 QEMU virtio-net 后端使用。

## 预期命令

```bash
sudo ip tuntap add tap0 mode tap user "$USER"
sudo ip link set tap0 up
```

## 维护的信息

无持久状态。脚本只创建临时网络设备。

## 错误处理

- `/dev/net/tun` 不存在：提示宿主环境不支持 TAP。
- `tap0` 已存在：可以复用或提示先删除。
- 权限不足：提示需要 sudo/CAP_NET_ADMIN。

## 与 packetfs 的关系

TAP 提供 host 到 QEMU guest 的 Ethernet frame 输入。

## 测试点

- `ip link show tap0` 能看到设备 up。
