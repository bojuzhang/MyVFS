# user/src/syscall.rs

## 额外说明

syscall inline asm 和基础 wrapper 可照搬 rCore。新增 mount/stat/getdents wrapper。

## 文件职责

提供用户态 syscall wrapper，让 demo 程序用普通函数调用文件系统 syscall。

## 需要提供的 wrapper

```rust
pub fn mount(fs_name: &str, target: &str, options: &str) -> isize;
pub fn umount(target: &str) -> isize;
pub fn open(path: &str, flags: u32) -> isize;
pub fn close(fd: usize) -> isize;
pub fn read(fd: usize, buf: &mut [u8]) -> isize;
pub fn write(fd: usize, buf: &[u8]) -> isize;
pub fn stat(path: &str, stat: &mut Stat) -> isize;
pub fn getdents(fd: usize, buf: &mut [u8]) -> isize;
```

## 与 packetfs 的关系

Demo 程序必须通过这些 wrapper 访问：

```text
/mnt/packetfs/packets
/mnt/packetfs/stats
```

不能直接调用 `submit_rx_frame()`。