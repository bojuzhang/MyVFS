# src/fs/packetfs/file.rs

## 文件职责

定义 `packetfs` 每次 open 后的 file 状态。这里实现 `/packets` 的阻塞 PCAP 流读取和 `/stats` 的文本快照读取。

## 需要定义的 enum

```rust
pub enum PacketFileKind {
    RootDir(RootDirFile),
    Packets(PacketCaptureFile),
    Stats(StatsFile),
}
```

## 需要定义的 struct

```rust
pub struct PacketCaptureFile {
    pub fs: Arc<PacketFsInner>,
    pub pcap_state: PcapStreamState,
    pub current_encoded: Option<Vec<u8>>,
    pub current_offset: usize,
    pub closed: AtomicBool,
}
```

```rust
pub struct StatsFile {
    pub fs: Arc<PacketFsInner>,
    pub snapshot_buf: Vec<u8>,
    pub offset: usize,
}
```

```rust
pub struct RootDirFile {
    pub fs: Arc<PacketFsInner>,
    pub offset: usize,
}
```

## 需要实现的 impl

- `impl File for PacketFileKind`
- `impl PacketCaptureFile`
  - `new(fs)`
  - `read_packets(buf)`
  - `close_reader()`
- `impl StatsFile`
  - `new(fs)`
  - `render_snapshot()`
- `impl RootDirFile`
  - `read_dir_entries()`

## `/packets` read 核心流程

1. 如果文件已关闭，返回 `EIO`。
2. 如果 PCAP global header 未输出，先输出 global header。
3. 如果 `current_encoded` 中还有未输出字节，继续拷贝。
4. 如果没有当前包，从 `PacketRing::pop_frame()` 取包。
5. 如果 ring 为空：
   - 如果 `mounted=false`，返回 `EIO`。
   - 否则把当前任务加入 wait queue 并阻塞。
   - 被唤醒后重新检查 ring。
6. 取到包后编码 PCAP record header + payload。
7. 支持小 buffer partial read。
8. 完整消费一个 record 后更新 stats。

## `/packets` close 流程

1. 如果已经 closed，直接返回。
2. 设置 closed。
3. 丢弃 `current_encoded`。
4. 设置 `reader_active=false`。
5. 唤醒可能等待状态变化的任务。

## `/stats` read 核心流程

1. open 时或第一次 read 时生成快照文本。
2. 根据 offset 拷贝到用户 buffer。
3. offset 到末尾后返回 0。
4. 不阻塞。

## write/seek 行为

- `write()` 全部返回 `EROFS`。
- `/packets` 的 `seek()` 返回 `ESPIPE`。
- `/stats` 可支持 seek 到开头，也可统一返回 `ESPIPE`；文档建议保持简单，返回 `ESPIPE`。