use crate::sync::{Mutex, MutexGuard, TryLockError};

#[derive(Debug)]
pub struct PacketStats {
    pub inner: Mutex<PacketStatsInner>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketStatsInner {
    pub captured_packets: u64,
    pub captured_bytes: u64,
    pub read_packets: u64,
    pub read_bytes: u64,
    pub queued_packets: u64,
    pub dropped_full: u64,
    pub dropped_inactive: u64,
    pub truncated_packets: u64,
    pub reader_active: bool,
    pub last_rx_ts: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatsSnapshot {
    pub captured_packets: u64,
    pub captured_bytes: u64,
    pub read_packets: u64,
    pub read_bytes: u64,
    pub queued_packets: u64,
    pub dropped_full: u64,
    pub dropped_inactive: u64,
    pub truncated_packets: u64,
    pub reader_active: bool,
    pub last_rx_ts: u64,
}

impl PacketStats {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(PacketStatsInner {
                captured_packets: 0,
                captured_bytes: 0,
                read_packets: 0,
                read_bytes: 0,
                queued_packets: 0,
                dropped_full: 0,
                dropped_inactive: 0,
                truncated_packets: 0,
                reader_active: false,
                last_rx_ts: 0,
            }),
        }
    }

    pub fn on_rx(&self, bytes: usize, timestamp_us: u64) {
        let mut stats = lock_unpoisoned(&self.inner);
        apply_rx(&mut stats, bytes, timestamp_us);
    }

    pub fn try_on_rx(&self, bytes: usize, timestamp_us: u64) -> bool {
        self.try_update(|stats| apply_rx(stats, bytes, timestamp_us))
    }

    pub fn on_read(&self, bytes: usize) {
        let mut stats = lock_unpoisoned(&self.inner);
        stats.read_packets = stats.read_packets.saturating_add(1);
        stats.read_bytes = stats.read_bytes.saturating_add(bytes as u64);
    }

    pub fn on_drop_full(&self) {
        let mut stats = lock_unpoisoned(&self.inner);
        apply_drop_full(&mut stats);
    }

    pub fn try_on_drop_full(&self) -> bool {
        self.try_update(apply_drop_full)
    }

    pub fn on_drop_inactive(&self) {
        let mut stats = lock_unpoisoned(&self.inner);
        apply_drop_inactive(&mut stats);
    }

    pub fn try_on_drop_inactive(&self) -> bool {
        self.try_update(apply_drop_inactive)
    }

    pub fn on_truncate(&self) {
        let mut stats = lock_unpoisoned(&self.inner);
        apply_truncate(&mut stats);
    }

    pub fn try_on_truncate(&self) -> bool {
        self.try_update(apply_truncate)
    }

    pub fn set_queued_packets(&self, queued: usize) {
        let mut stats = lock_unpoisoned(&self.inner);
        stats.queued_packets = queued as u64;
    }

    pub fn try_set_queued_packets(&self, queued: usize) -> bool {
        self.try_update(|stats| stats.queued_packets = queued as u64)
    }

    pub fn set_reader_active(&self, active: bool) {
        let mut stats = lock_unpoisoned(&self.inner);
        stats.reader_active = active;
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        let stats = lock_unpoisoned(&self.inner);
        StatsSnapshot {
            captured_packets: stats.captured_packets,
            captured_bytes: stats.captured_bytes,
            read_packets: stats.read_packets,
            read_bytes: stats.read_bytes,
            queued_packets: stats.queued_packets,
            dropped_full: stats.dropped_full,
            dropped_inactive: stats.dropped_inactive,
            truncated_packets: stats.truncated_packets,
            reader_active: stats.reader_active,
            last_rx_ts: stats.last_rx_ts,
        }
    }

    fn try_update(&self, update: impl FnOnce(&mut PacketStatsInner)) -> bool {
        match self.inner.try_lock() {
            Ok(mut stats) => {
                update(&mut stats);
                true
            }
            Err(TryLockError::Poisoned(err)) => {
                let mut stats = err.into_inner();
                update(&mut stats);
                true
            }
            Err(TryLockError::WouldBlock) => false,
        }
    }
}

impl Default for PacketStats {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsSnapshot {
    pub fn render_text(&self) -> Vec<u8> {
        format!(
            "filesystem=packetfs\n\
             mounted=true\n\
             captured_packets={}\n\
             captured_bytes={}\n\
             read_packets={}\n\
             read_bytes={}\n\
             queued_packets={}\n\
             dropped_full={}\n\
             dropped_inactive={}\n\
             truncated_packets={}\n\
             reader_active={}\n\
             last_rx_ts={}\n",
            self.captured_packets,
            self.captured_bytes,
            self.read_packets,
            self.read_bytes,
            self.queued_packets,
            self.dropped_full,
            self.dropped_inactive,
            self.truncated_packets,
            self.reader_active,
            self.last_rx_ts
        )
        .into_bytes()
    }
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

fn apply_rx(stats: &mut PacketStatsInner, bytes: usize, timestamp_us: u64) {
    stats.captured_packets = stats.captured_packets.saturating_add(1);
    stats.captured_bytes = stats.captured_bytes.saturating_add(bytes as u64);
    stats.last_rx_ts = timestamp_us;
}

fn apply_drop_full(stats: &mut PacketStatsInner) {
    stats.dropped_full = stats.dropped_full.saturating_add(1);
}

fn apply_drop_inactive(stats: &mut PacketStatsInner) {
    stats.dropped_inactive = stats.dropped_inactive.saturating_add(1);
}

fn apply_truncate(stats: &mut PacketStatsInner) {
    stats.truncated_packets = stats.truncated_packets.saturating_add(1);
}
