#[cfg(target_arch = "riscv64")]
use core::arch::asm;
#[cfg(not(target_arch = "riscv64"))]
use std::sync::Once;

pub const O_RDONLY: u32 = 0;
pub const O_WRONLY: u32 = 1;
pub const O_RDWR: u32 = 2;

pub const SYS_UMOUNT: usize = 39;
pub const SYS_MOUNT: usize = 40;
pub const SYS_OPEN: usize = 56;
pub const SYS_CLOSE: usize = 57;
pub const SYS_GETDENTS: usize = 61;
pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_STAT: usize = 80;
pub const SYS_EXIT: usize = 93;

const EINVAL: isize = -22;

#[cfg(not(target_arch = "riscv64"))]
fn ensure_host_kernel() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        if let Err(err) = kernel::fs::init() {
            std::eprintln!(
                "kernel host init failed: {} ({})",
                err.name(),
                err.as_isize()
            );
            std::process::exit(1);
        }
    });
}

#[cfg(not(target_arch = "riscv64"))]
fn syscall_host(id: usize, args: [usize; 3]) -> isize {
    if id == SYS_EXIT {
        std::process::exit(args[0] as i32);
    }
    ensure_host_kernel();
    kernel::syscall::syscall(id, args)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Stat {
    pub inode_id: u64,
    pub mode: u16,
    pub size: usize,
    pub file_type: u16,
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
fn syscall0(id: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") id,
            lateout("a0") ret,
        );
    }
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
fn syscall1(id: usize, arg0: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a7") id,
        );
    }
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
fn syscall2(id: usize, arg0: usize, arg1: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a7") id,
        );
    }
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
fn syscall3(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a7") id,
        );
    }
    ret
}

#[cfg(not(target_arch = "riscv64"))]
#[inline(always)]
fn syscall0(id: usize) -> isize {
    syscall_host(id, [0, 0, 0])
}

#[cfg(not(target_arch = "riscv64"))]
#[inline(always)]
fn syscall1(id: usize, arg0: usize) -> isize {
    syscall_host(id, [arg0, 0, 0])
}

#[cfg(not(target_arch = "riscv64"))]
#[inline(always)]
fn syscall2(id: usize, arg0: usize, arg1: usize) -> isize {
    syscall_host(id, [arg0, arg1, 0])
}

#[cfg(not(target_arch = "riscv64"))]
#[inline(always)]
fn syscall3(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    syscall_host(id, [arg0, arg1, arg2])
}

fn copy_cstr<const N: usize>(s: &str, buf: &mut [u8; N]) -> Result<*const u8, isize> {
    if s.len() + 1 > N || s.as_bytes().contains(&0) {
        return Err(EINVAL);
    }
    buf[..s.len()].copy_from_slice(s.as_bytes());
    buf[s.len()] = 0;
    Ok(buf.as_ptr())
}

pub fn mount(fs_name: &str, target: &str, options: &str) -> isize {
    let mut fs_name_buf = [0u8; 64];
    let mut target_buf = [0u8; 256];
    let mut options_buf = [0u8; 256];
    let fs_name_ptr = match copy_cstr(fs_name, &mut fs_name_buf) {
        Ok(ptr) => ptr,
        Err(err) => return err,
    };
    let target_ptr = match copy_cstr(target, &mut target_buf) {
        Ok(ptr) => ptr,
        Err(err) => return err,
    };
    let options_ptr = match copy_cstr(options, &mut options_buf) {
        Ok(ptr) => ptr,
        Err(err) => return err,
    };
    syscall3(
        SYS_MOUNT,
        fs_name_ptr as usize,
        target_ptr as usize,
        options_ptr as usize,
    )
}

pub fn umount(target: &str) -> isize {
    let mut target_buf = [0u8; 256];
    let target_ptr = match copy_cstr(target, &mut target_buf) {
        Ok(ptr) => ptr,
        Err(err) => return err,
    };
    syscall1(SYS_UMOUNT, target_ptr as usize)
}

pub fn open(path: &str, flags: u32) -> isize {
    let mut path_buf = [0u8; 256];
    let path_ptr = match copy_cstr(path, &mut path_buf) {
        Ok(ptr) => ptr,
        Err(err) => return err,
    };
    syscall2(SYS_OPEN, path_ptr as usize, flags as usize)
}

pub fn close(fd: usize) -> isize {
    syscall1(SYS_CLOSE, fd)
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    syscall3(SYS_READ, fd, buf.as_mut_ptr() as usize, buf.len())
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    syscall3(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len())
}

pub fn stat(path: &str, stat: &mut Stat) -> isize {
    let mut path_buf = [0u8; 256];
    let path_ptr = match copy_cstr(path, &mut path_buf) {
        Ok(ptr) => ptr,
        Err(err) => return err,
    };
    syscall2(SYS_STAT, path_ptr as usize, stat as *mut Stat as usize)
}

pub fn getdents(fd: usize, buf: &mut [u8]) -> isize {
    syscall3(SYS_GETDENTS, fd, buf.as_mut_ptr() as usize, buf.len())
}

#[cfg(target_arch = "riscv64")]
pub fn exit(code: i32) -> ! {
    let _ = syscall1(SYS_EXIT, code as usize);
    loop {}
}

#[cfg(not(target_arch = "riscv64"))]
pub fn exit(code: i32) -> ! {
    std::process::exit(code);
}

#[allow(dead_code)]
fn _keep_syscall0_checked() -> isize {
    syscall0(SYS_GETDENTS)
}
