# src/fs/packetfs/mod.rs

## 文件职责

作为 `packetfs` 模块入口，声明子模块、导出稳定 API。

## 需要声明的模块

```rust
pub mod api;
pub mod file;
pub mod fs;
pub mod inode;
pub mod pcap;
pub mod ring;
pub mod stats;
```

## 需要导出的接口

```rust
pub use api::{
    make_packetfs,
    prepare_default_mountpoint,
    submit_rx_frame,
    stats_snapshot,
    RxMeta,
    SubmitResult,
    DEFAULT_MOUNTPOINT,
    DEFAULT_PACKETS_PATH,
    DEFAULT_STATS_PATH,
};

pub use fs::{PacketFs, PacketFsConfig};
```

## 模块级不变量

- `packetfs` 是可挂载文件系统。
- 挂载后只有 `packets` 和 `stats` 两个文件。
- `/packets` 输出 PCAP。
- `/packets` 单读者。
- 队列满时丢新包。
- `submit_rx_frame()` 不睡眠。
- virtio-net 只提交 Ethernet frame。
