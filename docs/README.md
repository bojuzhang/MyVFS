# packetfs 文档总入口

## 项目目标

本项目设计一个运行在 rCore 风格教学内核中的可挂载 VFS 文件系统 `packetfs`。它把 网卡 RX 路径收到的原始 Ethernet frame 缓存在内核中，并通过普通文件接口暴露给用户程序：

```text
/mnt/packetfs/packets
/mnt/packetfs/stats
```

## 约束

- 当前内核 crate 允许使用 `std`，例如集合、`Arc`、`OnceLock`、时间和宿主 IO。
- 内核共享状态使用的锁必须来自 `kernel/src/sync/` 的自写实现；不要在 `kernel/src` 中新增 `std::sync::Mutex`、`RwLock` 或 `Condvar`。
- 当前默认实现是保留部分 `std` 能力的 host model，用于验证 packetfs/VFS/PCAP/stats 行为。
- QEMU/no_std RISC-V 真实内核是后续迁移目标；`scripts/run-qemu.sh --qemu` 保留旧 QEMU 启动路径，默认运行 host demo。
- `packetfs` 是可挂载文件系统，不是 `/dev/packet` 设备文件方案。
- 用户访问路径固定为 `/mnt/packetfs/packets` 和 `/mnt/packetfs/stats`。
- 不实现 TX 发包、socket、TCP/IP、ARP、ping。
- `/packets` 输出 classic PCAP 流。
- `/packets` 无包时阻塞等待。
- `/packets` 只允许单读者。
- 队列满时丢新包。
- `packetfs` 自己维护 packet queue。
- virtio-net 只负责调用 `packetfs::submit_rx_frame()`

## 实现目录

```
kernel/
  src/
    main.rs
    fs/
      mod.rs
      vfs.rs
      path.rs
      mount.rs
      fd.rs
      stat.rs
      ramfs.rs
      packetfs/
        mod.rs
        fs.rs
        inode.rs
        file.rs
        ring.rs
        pcap.rs
        stats.rs
        api.rs
    net/
      mod.rs
      virtio_net.rs
      netbuf.rs
    task/
      mod.rs
      wait.rs
    syscall/
      mod.rs
      fs.rs
    sync/
      mod.rs
user/
  src/
    bin/
      packetdump.rs
      packetstat.rs
      mount_packetfs.rs
scripts/
  run-qemu.sh
  setup-tap.sh
  send-frame.py
  collect-pcap.py
docs/
```

## 文档更新记录

文档一致性更新记录维护在 [CHANGELOG.md](CHANGELOG.md)。修改实现口径、公共接口或约束时，应同步更新对应模块文档和该记录。

## 明确不做

- 不做 `/dev/packet` 方案。
- 不做 packet image 文件系统。
- 不做多 reader。
- 不做 TX 写包。
- 不做 socket API。
- 不做 TCP/IP/ARP/ICMP。
- 不做权限模型、uid/gid、时间戳、硬链接。
- 不做 mount namespace、bind mount、page cache。
