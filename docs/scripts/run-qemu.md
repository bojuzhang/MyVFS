# scripts/run-qemu.sh

## 文件职责

默认启动 host model demo，运行 `packetdump` 并输出 PCAP hex 与 stats。传入 `--qemu` 或设置 `PACKETFS_DEMO_MODE=qemu` 时，保留旧 QEMU RISC-V 教学内核启动路径。

当前项目默认运行模型是保留部分 `std` 的 host model，不要求 QEMU、TAP 或 bare-metal kernel image。

## 预期命令

```bash
scripts/run-qemu.sh
```

legacy QEMU 路径：

```bash
qemu-system-riscv64 \
  -machine virt \
  -nographic \
  -kernel target/riscv64gc-unknown-none-elf/release/kernel \
  -device virtio-net-device,netdev=net0 \
  -netdev tap,id=net0,ifname=tap0,script=no,downscript=no
```

## 需要说明的参数

- `-machine virt`：RISC-V virt 平台。
- `-nographic`：串口作为控制台。
- `virtio-net-device`：guest 中的虚拟网卡。
- `-netdev tap`：host TAP 后端。

## 与 packetfs 的关系

host demo 会在进程内向 `virtio_net` 注入 3 帧示例 Ethernet frame，再通过 `/mnt/packetfs/packets` 多次读取 PCAP。QEMU 模式仍依赖 guest virtio-net RX 能收到 host frame。

## 测试点

- 挂载、目录项、写保护、单读者、umount 等 VFS 管理输出完整出现。
- `PCAP_BEGIN` 到 `PCAP_END` 之间能恢复合法 PCAP。
- stats 中 `captured_packets=3` 且 `read_packets=3`。
