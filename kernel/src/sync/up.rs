use crate::sync::{Mutex, MutexGuard};

pub struct UPSafeCell<T> {
    inner: Mutex<T>,
}

impl<T> UPSafeCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    pub fn exclusive_access(&self) -> MutexGuard<'_, T> {
        self.inner.lock().expect("UPSafeCell poisoned")
    }
}

unsafe impl<T: Send> Sync for UPSafeCell<T> {}
