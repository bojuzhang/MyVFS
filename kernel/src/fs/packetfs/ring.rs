use std::cmp::min;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketRecord {
    pub seq: u64,
    pub timestamp_us: u64,
    pub wire_len: usize,
    pub cap_len: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushOutcome {
    Queued,
    DroppedFull,
    Truncated,
}

#[derive(Debug)]
pub struct PacketRing {
    pub queue: VecDeque<PacketRecord>,
    pub capacity: usize,
    pub next_seq: u64,
}

impl PacketRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
            next_seq: 0,
        }
    }

    pub fn push_frame(&mut self, frame: &[u8], timestamp_us: u64, snaplen: usize) -> PushOutcome {
        if self.is_full() {
            return PushOutcome::DroppedFull;
        }

        let wire_len = frame.len();
        let cap_len = min(wire_len, snaplen);
        let outcome = if cap_len < wire_len {
            PushOutcome::Truncated
        } else {
            PushOutcome::Queued
        };

        let record = PacketRecord {
            seq: self.next_seq,
            timestamp_us,
            wire_len,
            cap_len,
            data: frame[..cap_len].to_vec(),
        };
        self.next_seq = self.next_seq.wrapping_add(1);
        self.queue.push_back(record);
        outcome
    }

    pub fn pop_frame(&mut self) -> Option<PacketRecord> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.capacity
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}
