use std::sync::Arc;

use super::error::{FsError, FsResult};
use super::fd::OpenFlags;
use super::stat::{DirEntry, Metadata};

pub type DynInode = Arc<dyn Inode + Send + Sync>;
pub type DynFile = Arc<dyn File + Send + Sync>;
pub type DynFileSystem = Arc<dyn FileSystem + Send + Sync>;

pub struct UserBuffer<'a> {
    pub buffers: Vec<&'a mut [u8]>,
}

impl<'a> UserBuffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buffers: vec![buf] }
    }

    pub fn len(&self) -> usize {
        self.buffers.iter().map(|buf| buf.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.buffers.iter().all(|buf| buf.is_empty())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len());
        for buf in &self.buffers {
            out.extend_from_slice(buf);
        }
        out
    }

    pub fn write_from_slice(&mut self, src: &[u8]) -> usize {
        let mut copied = 0;
        for dst in &mut self.buffers {
            if copied == src.len() {
                break;
            }
            let take = usize::min(dst.len(), src.len() - copied);
            dst[..take].copy_from_slice(&src[copied..copied + take]);
            copied += take;
        }
        copied
    }
}

impl<'a> From<&'a mut [u8]> for UserBuffer<'a> {
    fn from(buf: &'a mut [u8]) -> Self {
        Self::new(buf)
    }
}

pub trait FileSystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn mount(&self, options: &str) -> FsResult<DynInode>;
    fn umount(&self) -> FsResult<()> {
        Ok(())
    }
    fn root_inode(&self) -> DynInode;
}

pub trait Inode: Send + Sync {
    fn metadata(&self) -> FsResult<Metadata>;
    fn lookup(&self, name: &str) -> FsResult<DynInode>;
    fn readdir(&self) -> FsResult<Vec<DirEntry>>;
    fn mkdir(&self, _name: &str) -> FsResult<DynInode> {
        Err(FsError::Erofs)
    }
    fn open(&self, flags: OpenFlags) -> FsResult<DynFile>;

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> FsResult<usize>;
    fn write_at(&self, offset: usize, buf: &[u8]) -> FsResult<usize>;
}

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, offset: usize, buf: UserBuffer<'_>) -> FsResult<usize>;
    fn write(&self, offset: usize, buf: UserBuffer<'_>) -> FsResult<usize>;
    fn stat(&self) -> FsResult<Metadata>;
    fn close(&self) -> FsResult<()>;
    fn seek(&self, current_offset: usize, pos: SeekFrom) -> FsResult<usize>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SeekFrom {
    Start(usize),
    Current(isize),
    End(isize),
}
