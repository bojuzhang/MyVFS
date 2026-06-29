# packetfs 文档总入口

## 项目目标

本项目设计一个运行在 rCore 风格教学内核中的可挂载 VFS 文件系统 `packetfs`。它把 virtio-net 收到的原始 Ethernet frame 缓存在内核中，并通过普通文件接口暴露给用户程序：

```text
/mnt/packetfs/packets
/mnt/packetfs/stats
```

## 约束

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

## 明确不做

- 不做 `/dev/packet` 方案。
- 不做 packet image 文件系统。
- 不做多 reader。
- 不做 TX 写包。
- 不做 socket API。
- 不做 TCP/IP/ARP/ICMP。
- 不做权限模型、uid/gid、时间戳、硬链接。
- 不做 mount namespace、bind mount、page cache。