use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::fs::{DynInode, FileSystem, FsError, FsResult};
use crate::sync::{Mutex, MutexGuard, WaitQueue};

use super::api::set_active_instance;
use super::inode::PacketInode;
use super::ring::PacketRing;
use super::stats::PacketStats;

pub const DEFAULT_SNAPLEN: usize = 2048;
pub const DEFAULT_CAPACITY: usize = 256;
pub const MIN_SNAPLEN: usize = 64;
pub const MAX_SNAPLEN: usize = 4096;
pub const MIN_CAPACITY: usize = 1;
pub const MAX_CAPACITY: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketFsConfig {
    pub snaplen: usize,
    pub capacity: usize,
}

#[derive(Debug)]
pub struct PacketFs {
    pub inner: Arc<PacketFsInner>,
}

#[derive(Debug)]
pub struct PacketFsInner {
    pub config: PacketFsConfig,
    pub root_inode: Arc<PacketInode>,
    pub packets_inode: Arc<PacketInode>,
    pub stats_inode: Arc<PacketInode>,
    pub ring: Mutex<PacketRing>,
    pub stats: PacketStats,
    pub wait_queue: WaitQueue,
    pub reader_active: AtomicBool,
    pub mounted: AtomicBool,
}

impl PacketFsConfig {
    pub fn default() -> Self {
        Self {
            snaplen: DEFAULT_SNAPLEN,
            capacity: DEFAULT_CAPACITY,
        }
    }

    pub fn parse(options: &str) -> FsResult<Self> {
        let mut config = Self::default();
        let options = options.trim();
        if options.is_empty() {
            return Ok(config);
        }

        for raw_item in options.split(',') {
            let item = raw_item.trim();
            if item.is_empty() {
                continue;
            }

            let (key, value) = item.split_once('=').ok_or(FsError::Einval)?;
            let key = key.trim();
            let value = value.trim();
            if key.is_empty() || value.is_empty() {
                return Err(FsError::Einval);
            }

            let parsed = value.parse::<usize>().map_err(|_| FsError::Einval)?;
            match key {
                "snaplen" => config.snaplen = parsed,
                "capacity" => config.capacity = parsed,
                _ => return Err(FsError::Einval),
            }
        }

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> FsResult<()> {
        if !(MIN_SNAPLEN..=MAX_SNAPLEN).contains(&self.snaplen) {
            return Err(FsError::Einval);
        }
        if !(MIN_CAPACITY..=MAX_CAPACITY).contains(&self.capacity) {
            return Err(FsError::Einval);
        }
        Ok(())
    }
}

impl Default for PacketFsConfig {
    fn default() -> Self {
        Self {
            snaplen: DEFAULT_SNAPLEN,
            capacity: DEFAULT_CAPACITY,
        }
    }
}

impl PacketFs {
    pub fn new(config: PacketFsConfig) -> FsResult<Self> {
        config.validate()?;
        Ok(Self {
            inner: PacketFsInner::new(config, true),
        })
    }
}

impl FileSystem for PacketFs {
    fn name(&self) -> &'static str {
        "packetfs"
    }

    fn mount(&self, options: &str) -> FsResult<DynInode> {
        let config = if options.trim().is_empty() {
            self.inner.config
        } else {
            PacketFsConfig::parse(options)?
        };
        let mounted = PacketFs::new(config)?;
        set_active_instance(mounted.inner.clone())?;
        Ok(mounted.root_inode())
    }

    fn root_inode(&self) -> DynInode {
        self.inner.root_inode.clone()
    }
}

impl PacketFsInner {
    pub fn new(config: PacketFsConfig, mounted: bool) -> Arc<Self> {
        Arc::new_cyclic(|weak| {
            let root_inode = Arc::new(PacketInode::new_root_weak(weak.clone()));
            let packets_inode = Arc::new(PacketInode::new_packets_weak(weak.clone()));
            let stats_inode = Arc::new(PacketInode::new_stats_weak(weak.clone()));

            Self {
                config,
                root_inode,
                packets_inode,
                stats_inode,
                ring: Mutex::new(PacketRing::new(config.capacity)),
                stats: PacketStats::new(),
                wait_queue: WaitQueue::new(),
                reader_active: AtomicBool::new(false),
                mounted: AtomicBool::new(mounted),
            }
        })
    }

    pub fn is_mounted(&self) -> bool {
        self.mounted.load(Ordering::Acquire)
    }

    pub fn begin_umount(&self) -> FsResult<()> {
        if self.reader_active.load(Ordering::Acquire) {
            return Err(FsError::Ebusy);
        }
        self.mounted.store(false, Ordering::Release);
        {
            let mut ring = lock_unpoisoned(&self.ring);
            ring.clear();
            self.stats.set_queued_packets(0);
        }
        self.wait_queue.wake_all();
        Ok(())
    }
}

pub(crate) fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}
