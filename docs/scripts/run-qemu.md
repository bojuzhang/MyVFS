# scripts/run-qemu.sh

## 文件职责

启动 QEMU RISC-V 教学内核，并连接 virtio-net 到 host TAP。

当前项目默认运行模型是保留部分 `std` 的 host model。该脚本属于后续 QEMU/no_std 真实内核路径，要求已经存在可由 QEMU 加载的 RISC-V bare-metal kernel image。

## 预期命令

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

packetfs 本身不依赖 QEMU 参数，但 demo 需要 virtio-net RX 能收到 host frame。

## 测试点

- guest 启动日志能显示 virtio-net 初始化。
- packetfs mount 后 stats 可读。
