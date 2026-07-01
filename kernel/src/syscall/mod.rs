pub mod fs;
pub mod mm;

pub const SYS_MKDIR: usize = 34;
pub const SYS_OPEN: usize = 56;
pub const SYS_CLOSE: usize = 57;
pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_STAT: usize = 80;
pub const SYS_GETDENTS: usize = 61;
pub const SYS_MOUNT: usize = 40;
pub const SYS_UMOUNT: usize = 39;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYS_MKDIR => fs::sys_mkdir(args[0] as *const u8),
        SYS_OPEN => fs::sys_open(args[0] as *const u8, args[1] as u32),
        SYS_CLOSE => fs::sys_close(args[0]),
        SYS_READ => fs::sys_read(args[0], args[1] as *mut u8, args[2]),
        SYS_WRITE => fs::sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_STAT => fs::sys_stat(args[0] as *const u8, args[1] as *mut crate::fs::Stat),
        SYS_GETDENTS => fs::sys_getdents(args[0], args[1] as *mut u8, args[2]),
        SYS_MOUNT => fs::sys_mount(
            args[0] as *const u8,
            args[1] as *const u8,
            args[2] as *const u8,
        ),
        SYS_UMOUNT => fs::sys_umount(args[0] as *const u8),
        _ => -38,
    }
}
