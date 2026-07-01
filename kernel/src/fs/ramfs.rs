use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::sync::Mutex;

use super::error::{FsError, FsResult};
use super::fd::OpenFlags;
use super::stat::{DirEntry, FileType, Metadata};
use super::vfs::{DynFile, DynInode, File, FileSystem, Inode, SeekFrom, UserBuffer};

pub struct RamFs {
    pub root: Arc<RamInode>,
    pub next_inode_id: Arc<AtomicU64>,
}

pub struct RamInode {
    pub id: u64,
    pub name: String,
    pub kind: FileType,
    pub next_inode_id: Arc<AtomicU64>,
    pub inner: Arc<Mutex<RamInodeInner>>,
}

pub enum RamInodeInner {
    Directory(BTreeMap<String, DynInode>),
    File(Vec<u8>),
}

pub struct RamFile {
    pub inode: Arc<RamInode>,
    pub offset: Mutex<usize>,
    pub flags: OpenFlags,
}

impl RamFs {
    pub fn new() -> Self {
        let next_inode_id = Arc::new(AtomicU64::new(2));
        Self {
            root: Arc::new(RamInode {
                id: 1,
                name: "/".to_string(),
                kind: FileType::Directory,
                next_inode_id: next_inode_id.clone(),
                inner: Arc::new(Mutex::new(RamInodeInner::Directory(BTreeMap::new()))),
            }),
            next_inode_id,
        }
    }

    pub fn create_dir(&self, parent: &Arc<RamInode>, name: &str) -> FsResult<Arc<RamInode>> {
        self.create_inode(parent, name, FileType::Directory)
    }

    pub fn create_file(&self, parent: &Arc<RamInode>, name: &str) -> FsResult<Arc<RamInode>> {
        self.create_inode(parent, name, FileType::Regular)
    }

    fn create_inode(
        &self,
        parent: &Arc<RamInode>,
        name: &str,
        kind: FileType,
    ) -> FsResult<Arc<RamInode>> {
        parent.create_child(name, kind)
    }
}

impl Default for RamFs {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for RamFs {
    fn name(&self) -> &'static str {
        "ramfs"
    }

    fn mount(&self, _options: &str) -> FsResult<DynInode> {
        Ok(self.root_inode())
    }

    fn root_inode(&self) -> DynInode {
        self.root.clone()
    }
}

impl Inode for RamInode {
    fn metadata(&self) -> FsResult<Metadata> {
        let inner = self.inner.lock().map_err(|_| FsError::Eio)?;
        let size = match &*inner {
            RamInodeInner::Directory(children) => children.len(),
            RamInodeInner::File(data) => data.len(),
        };
        Ok(Metadata {
            inode_id: self.id,
            file_type: self.kind,
            mode: match self.kind {
                FileType::Directory => 0o555,
                FileType::Regular => 0o444,
                FileType::Pipe | FileType::CharDevice => 0o444,
            },
            size,
            nlink: 1,
        })
    }

    fn lookup(&self, name: &str) -> FsResult<DynInode> {
        if name.is_empty() || name == "." {
            return Err(FsError::Einval);
        }
        if name == ".." {
            return Err(FsError::Enoent);
        }

        let inner = self.inner.lock().map_err(|_| FsError::Eio)?;
        match &*inner {
            RamInodeInner::Directory(children) => {
                children.get(name).cloned().ok_or(FsError::Enoent)
            }
            RamInodeInner::File(_) => Err(FsError::Enotdir),
        }
    }

    fn readdir(&self) -> FsResult<Vec<DirEntry>> {
        let inner = self.inner.lock().map_err(|_| FsError::Eio)?;
        match &*inner {
            RamInodeInner::Directory(children) => {
                let mut entries = Vec::with_capacity(children.len() + 2);
                entries.push(DirEntry {
                    name: ".".to_string(),
                    inode_id: self.id,
                    file_type: FileType::Directory,
                });
                entries.push(DirEntry {
                    name: "..".to_string(),
                    inode_id: self.id,
                    file_type: FileType::Directory,
                });
                for (name, inode) in children {
                    let metadata = inode.metadata()?;
                    entries.push(DirEntry {
                        name: name.clone(),
                        inode_id: metadata.inode_id,
                        file_type: metadata.file_type,
                    });
                }
                Ok(entries)
            }
            RamInodeInner::File(_) => Err(FsError::Enotdir),
        }
    }

    fn mkdir(&self, name: &str) -> FsResult<DynInode> {
        Ok(self.create_child(name, FileType::Directory)?)
    }

    fn open(&self, flags: OpenFlags) -> FsResult<DynFile> {
        if flags.contains(OpenFlags::DIRECTORY) && self.kind != FileType::Directory {
            return Err(FsError::Enotdir);
        }
        if self.kind == FileType::Directory && flags.writable() {
            return Err(FsError::Eisdir);
        }

        Ok(Arc::new(RamFile {
            inode: Arc::new(self.clone_shallow()?),
            offset: Mutex::new(0),
            flags,
        }))
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> FsResult<usize> {
        let inner = self.inner.lock().map_err(|_| FsError::Eio)?;
        match &*inner {
            RamInodeInner::Directory(_) => Err(FsError::Eisdir),
            RamInodeInner::File(data) => {
                if offset >= data.len() {
                    return Ok(0);
                }
                let end = usize::min(offset + buf.len(), data.len());
                let len = end - offset;
                buf[..len].copy_from_slice(&data[offset..end]);
                Ok(len)
            }
        }
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> FsResult<usize> {
        let mut inner = self.inner.lock().map_err(|_| FsError::Eio)?;
        match &mut *inner {
            RamInodeInner::Directory(_) => Err(FsError::Eisdir),
            RamInodeInner::File(data) => {
                let end = offset.checked_add(buf.len()).ok_or(FsError::Enomem)?;
                if end > data.len() {
                    data.resize(end, 0);
                }
                data[offset..end].copy_from_slice(buf);
                Ok(buf.len())
            }
        }
    }
}

impl RamInode {
    fn create_child(&self, name: &str, kind: FileType) -> FsResult<Arc<RamInode>> {
        if name.is_empty() || name == "." || name == ".." || name.contains('/') {
            return Err(FsError::Einval);
        }

        let mut inner = self.inner.lock().map_err(|_| FsError::Eio)?;
        let children = match &mut *inner {
            RamInodeInner::Directory(children) => children,
            RamInodeInner::File(_) => return Err(FsError::Enotdir),
        };
        if children.contains_key(name) {
            return Err(FsError::Ebusy);
        }

        let inode = Arc::new(RamInode {
            id: self.next_inode_id.fetch_add(1, Ordering::Relaxed),
            name: name.to_string(),
            kind,
            next_inode_id: self.next_inode_id.clone(),
            inner: Arc::new(Mutex::new(match kind {
                FileType::Directory => RamInodeInner::Directory(BTreeMap::new()),
                FileType::Regular | FileType::Pipe | FileType::CharDevice => {
                    RamInodeInner::File(Vec::new())
                }
            })),
        });
        children.insert(name.to_string(), inode.clone() as DynInode);
        Ok(inode)
    }

    fn clone_shallow(&self) -> FsResult<Self> {
        Ok(Self {
            id: self.id,
            name: self.name.clone(),
            kind: self.kind,
            next_inode_id: self.next_inode_id.clone(),
            inner: self.inner.clone(),
        })
    }
}

impl File for RamFile {
    fn readable(&self) -> bool {
        self.flags.readable()
    }

    fn writable(&self) -> bool {
        self.flags.writable()
    }

    fn read(&self, mut buf: UserBuffer<'_>) -> FsResult<usize> {
        if !self.readable() {
            return Err(FsError::Eacces);
        }
        let mut offset = self.offset.lock().map_err(|_| FsError::Eio)?;
        let mut tmp = vec![0; buf.len()];
        let len = self.inode.read_at(*offset, &mut tmp)?;
        let copied = buf.write_from_slice(&tmp[..len]);
        *offset += copied;
        Ok(copied)
    }

    fn write(&self, buf: UserBuffer<'_>) -> FsResult<usize> {
        if !self.writable() {
            return Err(FsError::Eacces);
        }
        let mut offset = self.offset.lock().map_err(|_| FsError::Eio)?;
        let data = buf.to_vec();
        let len = self.inode.write_at(*offset, &data)?;
        *offset += len;
        Ok(len)
    }

    fn stat(&self) -> FsResult<Metadata> {
        self.inode.metadata()
    }

    fn close(&self) -> FsResult<()> {
        Ok(())
    }

    fn seek(&self, pos: SeekFrom) -> FsResult<usize> {
        let metadata = self.inode.metadata()?;
        let mut offset = self.offset.lock().map_err(|_| FsError::Eio)?;
        let next = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::Current(delta) => apply_delta(*offset, delta)?,
            SeekFrom::End(delta) => apply_delta(metadata.size, delta)?,
        };
        *offset = next;
        Ok(next)
    }
}

fn apply_delta(base: usize, delta: isize) -> FsResult<usize> {
    if delta >= 0 {
        base.checked_add(delta as usize).ok_or(FsError::Einval)
    } else {
        base.checked_sub(delta.unsigned_abs())
            .ok_or(FsError::Einval)
    }
}
