use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PidHandle(pub usize);

static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

pub fn pid_alloc() -> PidHandle {
    PidHandle(NEXT_PID.fetch_add(1, Ordering::Relaxed))
}
