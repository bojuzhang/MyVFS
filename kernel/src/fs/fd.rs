use std::ops::{BitAnd, BitOr, BitOrAssign};

use super::error::{FsError, FsResult};
use super::vfs::{DynFile, UserBuffer};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OpenFlags {
    bits: u32,
}

impl OpenFlags {
    pub const RDONLY: Self = Self { bits: 0 };
    pub const WRONLY: Self = Self { bits: 1 << 0 };
    pub const RDWR: Self = Self { bits: 1 << 1 };
    pub const CREATE: Self = Self { bits: 1 << 2 };
    pub const TRUNC: Self = Self { bits: 1 << 3 };
    pub const DIRECTORY: Self = Self { bits: 1 << 4 };
    pub const ALL: Self = Self {
        bits: Self::WRONLY.bits
            | Self::RDWR.bits
            | Self::CREATE.bits
            | Self::TRUNC.bits
            | Self::DIRECTORY.bits,
    };

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn from_bits(bits: u32) -> FsResult<Self> {
        Self { bits }.validate()
    }

    pub const fn from_bits_truncate(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn bits(self) -> u32 {
        self.bits
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    pub fn readable(&self) -> bool {
        self.contains(Self::RDWR) || !self.contains(Self::WRONLY)
    }

    pub fn writable(&self) -> bool {
        self.contains(Self::WRONLY) || self.contains(Self::RDWR)
    }

    pub fn validate(self) -> FsResult<Self> {
        if self.bits & !Self::ALL.bits != 0 {
            return Err(FsError::Einval);
        }
        if self.contains(Self::WRONLY) && self.contains(Self::RDWR) {
            return Err(FsError::Einval);
        }
        Ok(self)
    }
}

impl BitOr for OpenFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitOrAssign for OpenFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl BitAnd for OpenFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

#[derive(Clone)]
pub struct FileHandle {
    pub file: DynFile,
    pub flags: OpenFlags,
    pub offset: usize,
    pub debug_path: String,
}

impl FileHandle {
    pub fn read(&mut self, buf: UserBuffer<'_>) -> FsResult<usize> {
        self.flags.validate()?;
        if !self.flags.readable() || !self.file.readable() {
            return Err(FsError::Eacces);
        }
        let read = self.file.read(buf)?;
        self.advance_offset(read)?;
        Ok(read)
    }

    pub fn write(&mut self, buf: UserBuffer<'_>) -> FsResult<usize> {
        self.flags.validate()?;
        if !self.flags.writable() || !self.file.writable() {
            return Err(FsError::Eacces);
        }
        let written = self.file.write(buf)?;
        self.advance_offset(written)?;
        Ok(written)
    }

    pub fn advance_offset(&mut self, amount: usize) -> FsResult<usize> {
        self.offset = self.offset.checked_add(amount).ok_or(FsError::Einval)?;
        Ok(self.offset)
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }
}

#[derive(Clone, Default)]
pub struct FdTable {
    pub entries: Vec<Option<FileHandle>>,
}

impl FdTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn alloc(&mut self, handle: FileHandle) -> FsResult<usize> {
        handle.flags.validate()?;
        if let Some((fd, slot)) = self
            .entries
            .iter_mut()
            .enumerate()
            .find(|(_, entry)| entry.is_none())
        {
            *slot = Some(handle);
            return Ok(fd);
        }

        let fd = self.entries.len();
        self.entries.push(Some(handle));
        Ok(fd)
    }

    pub fn get(&self, fd: usize) -> FsResult<&FileHandle> {
        self.entries
            .get(fd)
            .and_then(Option::as_ref)
            .ok_or(FsError::Einval)
    }

    pub fn get_mut(&mut self, fd: usize) -> FsResult<&mut FileHandle> {
        self.entries
            .get_mut(fd)
            .and_then(Option::as_mut)
            .ok_or(FsError::Einval)
    }

    pub fn close(&mut self, fd: usize) -> FsResult<()> {
        let entry = self.entries.get_mut(fd).ok_or(FsError::Einval)?;
        let handle = entry.take().ok_or(FsError::Einval)?;
        handle.file.close()
    }

    pub fn dup(&mut self, fd: usize) -> FsResult<usize> {
        let handle = self.get(fd)?.clone();
        self.alloc(handle)
    }

    pub fn read(&mut self, fd: usize, buf: UserBuffer<'_>) -> FsResult<usize> {
        self.get_mut(fd)?.read(buf)
    }

    pub fn write(&mut self, fd: usize, buf: UserBuffer<'_>) -> FsResult<usize> {
        self.get_mut(fd)?.write(buf)
    }

    pub fn fork_clone(&self) -> Self {
        self.clone()
    }
}
