use core::fmt::{self, Write};

pub mod syscall;

pub use syscall::{close, exit, getdents, mount, open, read, stat, umount, write, Stat, O_RDONLY};

pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub struct Stderr;

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = write(STDERR, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    let _ = Stdout.write_fmt(args);
}

pub fn eprint(args: fmt::Arguments) {
    let _ = Stderr.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print(format_args!("\n"))
    };
    ($fmt:expr) => {
        $crate::print(format_args!(concat!($fmt, "\n")))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print(format_args!(concat!($fmt, "\n"), $($arg)*))
    };
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        $crate::eprint(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::eprint(format_args!("\n"))
    };
    ($fmt:expr) => {
        $crate::eprint(format_args!(concat!($fmt, "\n")))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::eprint(format_args!(concat!($fmt, "\n"), $($arg)*))
    };
}
