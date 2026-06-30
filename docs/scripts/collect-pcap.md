# scripts/collect-pcap.py

## 文件职责

从 demo 日志中提取 `packetdump` 输出的 PCAP hex，恢复为 `cap.pcap` 或指定输出路径，并可解码展示写入文件中的 record 内容。

## 输入格式

```text
PCAP_BEGIN
<hex line>
<hex line>
PCAP_END
```

## 脚本流程

1. 读取日志文件或 stdin。
2. 找到 `PCAP_BEGIN`。
3. 收集直到 `PCAP_END` 的 hex 字符。
4. 去掉空白和非 hex 字符。
5. 写入二进制 `cap.pcap`。
6. 可选检查 record 数量和 payload 子串。
7. 可选输出 PCAP 文件摘要，展示 snaplen、network、record 数量和每条 Ethernet frame 的 header/payload。
8. 可选调用 `tcpdump -r cap.pcap`。

## 常用参数

- `--summary`：打印写入 PCAP 文件中的 record 摘要。
- `--expect-records N`：要求 PCAP 中有 `N` 条 record。
- `--expect-payload TEXT`：要求至少一条 record frame 中包含该 payload 子串，可重复使用。

## 错误处理

- 找不到 begin/end：返回失败。
- hex 长度为奇数：返回失败。
- PCAP magic 不正确：提示 packetdump 或 pcap encoder 有问题。
- record header/payload 截断：返回失败。
- record 数量或 payload 期望不满足：返回失败。

## 测试点

- 能把 demo 输出恢复为合法 pcap。
- 能检查 3 条 demo record 和对应 payload。
- `--summary` 能展示写入文件内容。
- `tcpdump -r cap.pcap` 能识别。
