use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::fs::{DirEntry, File, FileType, FsError, FsResult, Metadata, SeekFrom, UserBuffer};
use crate::sync::Mutex;

use super::fs::{lock_unpoisoned, PacketFsInner};
use super::pcap::{encode_global_header, encode_record, PcapStreamState};
use super::ring::PacketRecord;

#[derive(Debug)]
pub enum PacketFileKind {
    RootDir(RootDirFile),
    Packets(PacketCaptureFile),
    Stats(StatsFile),
}

#[derive(Debug)]
pub struct PacketCaptureFile {
    pub fs: Arc<PacketFsInner>,
    pub pcap_state: PcapStreamState,
    pub current_encoded: Option<Vec<u8>>,
    pub current_offset: usize,
    pub closed: AtomicBool,
    current_record_bytes: usize,
}

#[derive(Debug)]
pub struct StatsFile {
    pub fs: Arc<PacketFsInner>,
    pub snapshot_buf: Vec<u8>,
    pub offset: usize,
}

#[derive(Debug)]
pub struct RootDirFile {
    pub fs: Arc<PacketFsInner>,
    pub offset: usize,
}

impl File for Mutex<PacketFileKind> {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, _offset: usize, buf: UserBuffer<'_>) -> FsResult<usize> {
        let mut file = lock_unpoisoned(self);
        match &mut *file {
            PacketFileKind::RootDir(root) => root.read_dir_entries(buf),
            PacketFileKind::Packets(packets) => packets.read_packets(buf),
            PacketFileKind::Stats(stats) => stats.read(buf),
        }
    }

    fn write(&self, _offset: usize, _buf: UserBuffer<'_>) -> FsResult<usize> {
        Err(FsError::Erofs)
    }

    fn stat(&self) -> FsResult<Metadata> {
        let file = lock_unpoisoned(self);
        match &*file {
            PacketFileKind::RootDir(_) => Ok(Metadata {
                inode_id: 1,
                file_type: FileType::Directory,
                mode: 0o555,
                size: 0,
                nlink: 2,
            }),
            PacketFileKind::Packets(_) => Ok(Metadata {
                inode_id: 2,
                file_type: FileType::Regular,
                mode: 0o444,
                size: 0,
                nlink: 1,
            }),
            PacketFileKind::Stats(stats) => Ok(Metadata {
                inode_id: 3,
                file_type: FileType::Regular,
                mode: 0o444,
                size: stats.snapshot_buf.len(),
                nlink: 1,
            }),
        }
    }

    fn close(&self) -> FsResult<()> {
        let mut file = lock_unpoisoned(self);
        if let PacketFileKind::Packets(packets) = &mut *file {
            packets.close_reader();
        }
        Ok(())
    }

    fn seek(&self, _current_offset: usize, _pos: SeekFrom) -> FsResult<usize> {
        Err(FsError::Espipe)
    }
}

impl PacketCaptureFile {
    pub fn new(fs: Arc<PacketFsInner>) -> Self {
        Self {
            fs,
            pcap_state: PcapStreamState::default(),
            current_encoded: None,
            current_offset: 0,
            closed: AtomicBool::new(false),
            current_record_bytes: 0,
        }
    }

    pub fn read_packets(&mut self, mut buf: UserBuffer<'_>) -> FsResult<usize> {
        if self.closed.load(Ordering::Acquire) {
            return Err(FsError::Eio);
        }

        let capacity = buf.len();
        if capacity == 0 {
            return Ok(0);
        }

        let mut out = Vec::with_capacity(capacity);
        while out.len() < capacity {
            if self.current_encoded.is_none() {
                if !self.pcap_state.global_header_done {
                    self.current_encoded =
                        Some(encode_global_header(self.fs.config.snaplen).to_vec());
                    self.current_offset = 0;
                    self.current_record_bytes = 0;
                    self.pcap_state.global_header_done = true;
                } else {
                    if !out.is_empty() {
                        break;
                    }
                    let record = self.pop_frame_blocking()?;
                    self.current_record_bytes = record.cap_len;
                    self.current_encoded = Some(encode_record(&record));
                    self.current_offset = 0;
                }
            }

            let current = self.current_encoded.as_ref().ok_or(FsError::Eio)?;
            let remaining_record = current.len().saturating_sub(self.current_offset);
            if remaining_record == 0 {
                self.finish_current_record();
                continue;
            }

            let remaining_user = capacity - out.len();
            let take = remaining_record.min(remaining_user);
            out.extend_from_slice(&current[self.current_offset..self.current_offset + take]);
            self.current_offset += take;

            if self.current_offset == current.len() {
                self.finish_current_record();
            }
        }

        Ok(buf.write_from_slice(&out))
    }

    pub fn close_reader(&mut self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        self.current_encoded = None;
        self.current_offset = 0;
        self.current_record_bytes = 0;
        self.fs.reader_active.store(false, Ordering::Release);
        self.fs.stats.set_reader_active(false);
        self.fs.wait_queue.wake_all();
    }

    fn finish_current_record(&mut self) {
        self.current_encoded = None;
        self.current_offset = 0;
        if self.current_record_bytes > 0 {
            self.fs.stats.on_read(self.current_record_bytes);
            self.current_record_bytes = 0;
        }
    }

    fn pop_frame_blocking(&self) -> FsResult<PacketRecord> {
        loop {
            let wait_guard = self.fs.wait_queue.prepare_wait();
            {
                let mut ring = lock_unpoisoned(&self.fs.ring);
                if let Some(record) = ring.pop_frame() {
                    let queued = ring.len();
                    drop(ring);
                    self.fs.stats.set_queued_packets(queued);
                    drop(wait_guard);
                    return Ok(record);
                }
            }

            if !self.fs.is_mounted() {
                drop(wait_guard);
                return Err(FsError::Eio);
            }

            self.fs.wait_queue.sleep_current_with_guard(wait_guard);
        }
    }
}

impl Drop for PacketCaptureFile {
    fn drop(&mut self) {
        self.close_reader();
    }
}

impl StatsFile {
    pub fn new(fs: Arc<PacketFsInner>) -> Self {
        let snapshot_buf = fs.stats.snapshot().render_text();
        Self {
            fs,
            snapshot_buf,
            offset: 0,
        }
    }

    pub fn render_snapshot(&self) -> Vec<u8> {
        self.fs.stats.snapshot().render_text()
    }

    pub fn read(&mut self, mut buf: UserBuffer<'_>) -> FsResult<usize> {
        if self.offset >= self.snapshot_buf.len() {
            return Ok(0);
        }

        let capacity = buf.len();
        let end = (self.offset + capacity).min(self.snapshot_buf.len());
        let copied = buf.write_from_slice(&self.snapshot_buf[self.offset..end]);
        self.offset += copied;
        Ok(copied)
    }
}

impl RootDirFile {
    pub fn new(fs: Arc<PacketFsInner>) -> Self {
        Self { fs, offset: 0 }
    }

    pub fn read_dir_entries(&mut self, _buf: UserBuffer<'_>) -> FsResult<usize> {
        Err(FsError::Eisdir)
    }

    pub fn entries(&self) -> Vec<DirEntry> {
        vec![
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
        ]
    }
}
