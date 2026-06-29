# src/fs/packetfs/pcap.rs

## 文件职责

把 `PacketRecord` 编码成 classic PCAP 字节流，使 host 可以还原为 `.pcap` 文件并用 Wireshark 或 `tcpdump -r` 验证。

## 常量

```rust
pub const PCAP_MAGIC: u32 = 0xa1b2c3d4;
pub const PCAP_VERSION_MAJOR: u16 = 2;
pub const PCAP_VERSION_MINOR: u16 = 4;
pub const LINKTYPE_ETHERNET: u32 = 1;
```

所有字段使用 little-endian 输出。

## 需要定义的 struct

```rust
pub struct PcapGlobalHeader {
    pub magic_number: u32,
    pub version_major: u16,
    pub version_minor: u16,
    pub thiszone: i32,
    pub sigfigs: u32,
    pub snaplen: u32,
    pub network: u32,
}
```

```rust
pub struct PcapRecordHeader {
    pub ts_sec: u32,
    pub ts_usec: u32,
    pub incl_len: u32,
    pub orig_len: u32,
}
```

```rust
pub struct PcapStreamState {
    pub global_header_done: bool,
}
```

## 需要实现的接口

```rust
pub fn encode_global_header(snaplen: usize) -> [u8; 24];
pub fn encode_record_header(record: &PacketRecord) -> [u8; 16];
pub fn encode_record(record: &PacketRecord) -> Vec<u8>;
```

## 编码规则

global header：

```text
magic_number  = 0xa1b2c3d4
version_major = 2
version_minor = 4
thiszone      = 0
sigfigs       = 0
snaplen       = mount snaplen
network       = 1
```

record header：

```text
ts_sec   = timestamp_us / 1_000_000
ts_usec  = timestamp_us % 1_000_000
incl_len = cap_len
orig_len = wire_len
```

## partial read 要求

`pcap.rs` 只负责编码，`file.rs` 负责保存 `current_encoded` 和 `current_offset`。但是 `pcap.rs` 文档必须提醒：不能假设用户 buffer 足够大，一条 PCAP record 可能要分多次 read 输出。

## 不包含内容

- 不包含 virtio-net header。
- 不包含 Ethernet FCS。
- 不做 IP/TCP 解析。

## 测试点

- global header 字节长度 24。
- record header 字节长度 16。
- `tcpdump -r` 能识别输出文件。
