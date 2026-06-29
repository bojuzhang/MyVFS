# src/syscall/mm.rs 或用户缓冲辅助

## 额外说明

照搬 rCore。

## 文件职责

提供用户地址空间到内核可访问缓冲区的翻译。`packetfs` 不直接处理页表，只接收 syscall 层传来的 `UserBuffer`。

## 需要提供的接口

```rust
pub fn translated_str(token: usize, ptr: *const u8) -> FsResult<String>;
pub fn translated_byte_buffer(token: usize, ptr: *mut u8, len: usize) -> FsResult<UserBuffer>;
```

`UserBuffer` 应能被 `File::read()` 安全写入。

## packetfs 使用方式

```text
sys_read
 -> translated_byte_buffer
 -> file.read(UserBuffer)
 -> PacketCaptureFile writes PCAP bytes
```

## 与 rCore 的关系

直接照搬 rCore 的用户缓冲区翻译思想。packetfs 文档只需要说明它依赖这个接口，不重新设计页表。

## 测试点

- 用户 buffer 跨页时仍可写入。
- 非法指针返回错误。
