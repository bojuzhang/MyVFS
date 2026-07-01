use std::sync::{Arc, OnceLock};

use crate::fs::{self, FileSystem, FileType, FsError, FsResult};
use crate::sync::{Mutex, TryLockError};

use super::fs::{lock_unpoisoned, PacketFs, PacketFsConfig, PacketFsInner};
use super::ring::PushOutcome;
use super::stats::StatsSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxMeta {
    pub timestamp_us: u64,
    pub iface_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitResult {
    Queued,
    DroppedInactive,
    DroppedFull,
    Truncated,
}

pub const DEFAULT_MOUNTPOINT: &str = "/mnt/packetfs";
pub const DEFAULT_PACKETS_PATH: &str = "/mnt/packetfs/packets";
pub const DEFAULT_STATS_PATH: &str = "/mnt/packetfs/stats";

static ACTIVE_PACKETFS: OnceLock<Mutex<Option<Arc<PacketFsInner>>>> = OnceLock::new();

pub fn make_packetfs(config: PacketFsConfig) -> FsResult<Arc<dyn FileSystem>> {
    Ok(Arc::new(PacketFs::new(config)?))
}

pub fn prepare_default_mountpoint() -> FsResult<()> {
    ensure_dir_tree(DEFAULT_MOUNTPOINT)
}

pub fn submit_rx_frame(frame: &[u8], meta: RxMeta) -> SubmitResult {
    let Some(inner) = active_instance_nonblocking() else {
        return SubmitResult::DroppedInactive;
    };

    if !inner.is_mounted() {
        inner.stats.try_on_drop_inactive();
        return SubmitResult::DroppedInactive;
    }

    // RX must not sleep in this teaching model. If the ring spin lock is busy,
    // use try_lock and account it as capture-side backpressure/drop.
    let (outcome, queued) = match inner.ring.try_lock() {
        Ok(mut ring) => {
            let outcome = ring.push_frame(frame, meta.timestamp_us, inner.config.snaplen);
            (outcome, ring.len())
        }
        Err(TryLockError::Poisoned(err)) => {
            let mut ring = err.into_inner();
            let outcome = ring.push_frame(frame, meta.timestamp_us, inner.config.snaplen);
            (outcome, ring.len())
        }
        Err(TryLockError::WouldBlock) => {
            inner.stats.try_on_drop_full();
            return SubmitResult::DroppedFull;
        }
    };
    inner.stats.try_set_queued_packets(queued);

    match outcome {
        PushOutcome::Queued => {
            inner.stats.try_on_rx(frame.len(), meta.timestamp_us);
            inner.wait_queue.wake_one();
            SubmitResult::Queued
        }
        PushOutcome::Truncated => {
            inner.stats.try_on_rx(frame.len(), meta.timestamp_us);
            inner.stats.try_on_truncate();
            inner.wait_queue.wake_one();
            SubmitResult::Truncated
        }
        PushOutcome::DroppedFull => {
            inner.stats.try_on_drop_full();
            SubmitResult::DroppedFull
        }
    }
}

pub fn stats_snapshot() -> FsResult<StatsSnapshot> {
    let inner = active_instance().ok_or(FsError::Enodev)?;
    Ok(inner.stats.snapshot())
}

pub fn set_active_instance(inner: Arc<PacketFsInner>) -> FsResult<()> {
    let slot = ACTIVE_PACKETFS.get_or_init(|| Mutex::new(None));
    let mut active = lock_unpoisoned(slot);

    if active
        .as_ref()
        .is_some_and(|current| current.is_mounted() && !Arc::ptr_eq(current, &inner))
    {
        return Err(FsError::Ebusy);
    }

    *active = Some(inner);
    Ok(())
}

pub fn clear_active_instance() {
    if let Some(slot) = ACTIVE_PACKETFS.get() {
        let mut active = lock_unpoisoned(slot);
        *active = None;
    }
}

pub fn begin_active_umount() -> FsResult<()> {
    let inner = active_instance().ok_or(FsError::Enodev)?;
    inner.begin_umount()?;
    clear_active_instance();
    Ok(())
}

fn active_instance() -> Option<Arc<PacketFsInner>> {
    ACTIVE_PACKETFS
        .get()
        .and_then(|slot| lock_unpoisoned(slot).clone())
}

fn active_instance_nonblocking() -> Option<Arc<PacketFsInner>> {
    let slot = ACTIVE_PACKETFS.get()?;
    match slot.try_lock() {
        Ok(active) => active.clone(),
        Err(TryLockError::Poisoned(err)) => err.into_inner().clone(),
        Err(TryLockError::WouldBlock) => None,
    }
}

fn ensure_dir_tree(path: &str) -> FsResult<()> {
    let parsed = fs::path::Path::parse(path)?;
    if !parsed.is_absolute {
        return Err(FsError::Einval);
    }

    let mut current = String::new();
    for component in parsed.components {
        current.push('/');
        current.push_str(&component);
        ensure_dir(&current)?;
    }
    Ok(())
}

fn ensure_dir(path: &str) -> FsResult<()> {
    match fs::mkdir_path(path) {
        Ok(()) => Ok(()),
        Err(FsError::Ebusy) => match fs::stat_path(path)?.file_type {
            FileType::Directory => Ok(()),
            _ => Err(FsError::Enotdir),
        },
        Err(err) => Err(err),
    }
}
