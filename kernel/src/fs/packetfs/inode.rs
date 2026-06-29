use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use crate::fs::{
    DirEntry, DynFile, DynInode, FileType, FsError, FsResult, Inode, Metadata, OpenFlags,
};
use crate::sync::Mutex;

use super::file::{PacketCaptureFile, PacketFileKind, RootDirFile, StatsFile};
use super::fs::PacketFsInner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketNodeKind {
    Root,
    Packets,
    Stats,
}

#[derive(Debug)]
pub struct PacketInode {
    pub fs: Weak<PacketFsInner>,
    pub kind: PacketNodeKind,
    pub inode_id: u64,
}

impl PacketInode {
    pub fn new_root(fs: Arc<PacketFsInner>) -> Self {
        Self::new_root_weak(Arc::downgrade(&fs))
    }

    pub fn new_packets(fs: Arc<PacketFsInner>) -> Self {
        Self::new_packets_weak(Arc::downgrade(&fs))
    }

    pub fn new_stats(fs: Arc<PacketFsInner>) -> Self {
        Self::new_stats_weak(Arc::downgrade(&fs))
    }

    pub(crate) fn new_root_weak(fs: Weak<PacketFsInner>) -> Self {
        Self {
            fs,
            kind: PacketNodeKind::Root,
            inode_id: 1,
        }
    }

    pub(crate) fn new_packets_weak(fs: Weak<PacketFsInner>) -> Self {
        Self {
            fs,
            kind: PacketNodeKind::Packets,
            inode_id: 2,
        }
    }

    pub(crate) fn new_stats_weak(fs: Weak<PacketFsInner>) -> Self {
        Self {
            fs,
            kind: PacketNodeKind::Stats,
            inode_id: 3,
        }
    }

    fn fs(&self) -> FsResult<Arc<PacketFsInner>> {
        self.fs.upgrade().ok_or(FsError::Eio)
    }
}

impl Inode for PacketInode {
    fn metadata(&self) -> FsResult<Metadata> {
        let (file_type, mode, size, nlink) = match self.kind {
            PacketNodeKind::Root => (FileType::Directory, 0o555, 0, 2),
            PacketNodeKind::Packets => (FileType::Regular, 0o444, 0, 1),
            PacketNodeKind::Stats => {
                let size = self.fs()?.stats.snapshot().render_text().len();
                (FileType::Regular, 0o444, size, 1)
            }
        };

        Ok(Metadata {
            inode_id: self.inode_id,
            file_type,
            mode,
            size,
            nlink,
        })
    }

    fn lookup(&self, name: &str) -> FsResult<DynInode> {
        let fs = self.fs()?;
        match self.kind {
            PacketNodeKind::Root => match name {
                "." | "" => Ok(fs.root_inode.clone()),
                ".." => Ok(fs.root_inode.clone()),
                "packets" => Ok(fs.packets_inode.clone()),
                "stats" => Ok(fs.stats_inode.clone()),
                _ => Err(FsError::Enoent),
            },
            PacketNodeKind::Packets | PacketNodeKind::Stats => Err(FsError::Enotdir),
        }
    }

    fn readdir(&self) -> FsResult<Vec<DirEntry>> {
        let fs = self.fs()?;
        match self.kind {
            PacketNodeKind::Root => Ok(vec![
                DirEntry {
                    name: ".".to_string(),
                    inode_id: 1,
                    file_type: FileType::Directory,
                },
                DirEntry {
                    name: "..".to_string(),
                    inode_id: 1,
                    file_type: FileType::Directory,
                },
                DirEntry {
                    name: "packets".to_string(),
                    inode_id: 2,
                    file_type: FileType::Regular,
                },
                DirEntry {
                    name: "stats".to_string(),
                    inode_id: 3,
                    file_type: FileType::Regular,
                },
            ]),
            PacketNodeKind::Packets | PacketNodeKind::Stats => {
                drop(fs);
                Err(FsError::Enotdir)
            }
        }
    }

    fn open(&self, flags: OpenFlags) -> FsResult<DynFile> {
        let fs = self.fs()?;
        if !fs.is_mounted() {
            return Err(FsError::Eio);
        }

        match self.kind {
            PacketNodeKind::Root => {
                if flags.writable() {
                    return Err(FsError::Eisdir);
                }
                Ok(Arc::new(Mutex::new(PacketFileKind::RootDir(
                    RootDirFile::new(fs),
                ))))
            }
            PacketNodeKind::Packets => {
                if flags.writable() {
                    return Err(FsError::Eacces);
                }
                if fs
                    .reader_active
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    return Err(FsError::Ebusy);
                }
                fs.stats.set_reader_active(true);
                Ok(Arc::new(Mutex::new(PacketFileKind::Packets(
                    PacketCaptureFile::new(fs),
                ))))
            }
            PacketNodeKind::Stats => {
                if flags.writable() {
                    return Err(FsError::Eacces);
                }
                Ok(Arc::new(Mutex::new(PacketFileKind::Stats(StatsFile::new(
                    fs,
                )))))
            }
        }
    }

    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> FsResult<usize> {
        match self.kind {
            PacketNodeKind::Root => Err(FsError::Eisdir),
            PacketNodeKind::Packets | PacketNodeKind::Stats => Err(FsError::Espipe),
        }
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> FsResult<usize> {
        Err(FsError::Erofs)
    }
}
