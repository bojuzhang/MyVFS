# src/syscall/mod.rs

## 额外说明

照搬 rCore，少量改写。

## 文件职责

负责 syscall 编号分发，把用户态 trap 传入的 syscall id 和参数路由到具体实现。

## 需要维护的信息

- syscall number 常量。
- 分发 match。
- 参数数量和顺序约定。

## 需要新增或确认的 syscall

```text
SYS_OPEN
SYS_CLOSE
SYS_READ
SYS_WRITE
SYS_MKDIR
SYS_STAT
SYS_GETDENTS
SYS_MOUNT
SYS_UMOUNT
```

其中 `SYS_MOUNT`、`SYS_UMOUNT`、`SYS_MKDIR`、`SYS_STAT`、`SYS_GETDENTS` 若 rCore 基线没有，需要新增。本项目内 `SYS_MKDIR = 34`。

## 核心流程

```rust
match syscall_id {
    SYS_MKDIR => fs::sys_mkdir(args[0]),
    SYS_OPEN => fs::sys_open(args[0], args[1]),
    SYS_READ => fs::sys_read(args[0], args[1], args[2]),
    ...
}
```
