use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::packetfs::api::{submit_rx_frame, RxMeta, SubmitResult};
use crate::sync::Mutex;

#[derive(Debug, Clone)]
struct RxFrame {
    iface_id: u32,
    frame: Vec<u8>,
}

pub struct VirtioNet {
    iface_id: u32,
}

impl VirtioNet {
    pub fn new(iface_id: u32) -> Self {
        Self { iface_id }
    }

    pub fn submit_rx_frame(&self, ethernet_frame: &[u8]) -> SubmitResult {
        submit_rx_frame(
            ethernet_frame,
            RxMeta {
                timestamp_us: timestamp_us(),
                iface_id: self.iface_id,
            },
        )
    }

    pub fn enqueue_rx_frame(&self, ethernet_frame: &[u8]) {
        enqueue_rx_frame_on(self.iface_id, ethernet_frame);
    }
}

pub fn poll_rx() -> Option<SubmitResult> {
    let rx = rx_queue();
    let frame = rx.lock().ok()?.pop_front()?;
    Some(submit_rx_frame(
        &frame.frame,
        RxMeta {
            timestamp_us: timestamp_us(),
            iface_id: frame.iface_id,
        },
    ))
}

pub fn enqueue_rx_frame(frame: &[u8]) {
    enqueue_rx_frame_on(0, frame);
}

pub fn enqueue_rx_frame_on(iface_id: u32, frame: &[u8]) {
    if let Ok(mut rx) = rx_queue().lock() {
        rx.push_back(RxFrame {
            iface_id,
            frame: frame.to_vec(),
        });
    }
}

pub fn inject_rx_frame_for_test(frame: &[u8]) -> SubmitResult {
    enqueue_rx_frame(frame);
    poll_rx().unwrap_or(SubmitResult::DroppedInactive)
}

fn timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros() as u64)
        .unwrap_or(0)
}

fn rx_queue() -> &'static Mutex<VecDeque<RxFrame>> {
    static RX_QUEUE: OnceLock<Mutex<VecDeque<RxFrame>>> = OnceLock::new();
    RX_QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}
