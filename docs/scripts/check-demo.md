# scripts/check-demo.sh

## 文件职责

运行 host demo 并自动检查 packetfs 输出。设置 `CHECK_DEMO_RUN=0` 时，也可以只检查已有日志。该脚本只检查运行结果，不参与 packetfs 核心实现。

## 检查项

- 输出是否包含 packetfs mount 成功。
- 输出是否包含 `/mnt/packetfs/packets` open 成功。
- 输出是否包含 `PCAP_BEGIN` 和 `PCAP_END`。
- collect 后的 `target/cap.pcap` 是否存在。
- 如果环境中存在 `tcpdump`，检查 `tcpdump -r target/cap.pcap` 是否成功。
- stats 是否显示 `captured_packets > 0`。

## 失败排查提示

- 没有包：检查 host demo 注入路径和 `virtio_net::poll_rx()`。
- PCAP 不合法：检查 `pcap.rs` header 编码。
- read 不返回：检查 wait queue 和 submit wakeup。

## 与 packetfs 的关系

该脚本只做运行结果检查，不参与内核功能。

## 测试点

- 成功 demo 返回 0。
- 缺少 PCAP 输出时返回非 0。
