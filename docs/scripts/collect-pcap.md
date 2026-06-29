# scripts/collect-pcap.py

## 文件职责

从 QEMU 串口日志中提取 `packetdump` 输出的 PCAP hex，恢复为 `cap.pcap`。

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
6. 可选调用 `tcpdump -r cap.pcap`。

## 错误处理

- 找不到 begin/end：返回失败。
- hex 长度为奇数：返回失败。
- PCAP magic 不正确：提示 packetdump 或 pcap encoder 有问题。

## 测试点

- 能把 demo 输出恢复为合法 pcap。
- `tcpdump -r cap.pcap` 能识别。
