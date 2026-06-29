本项目试图实现一个基于 rCore 和 QEMU 实现的直接读取网卡包的 VFS。

但当前实现采用保留部分 `std` 能力的 host model，用来集中验证 VFS、packetfs、PCAP 输出、统计信息和阻塞读模型。QEMU/no_std RISC-V 真实内核是后续迁移目标。