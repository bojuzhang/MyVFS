use core::cell::UnsafeCell;
use core::fmt;
use core::hint::spin_loop;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub type Mutex<T> = SpinMutex<T>;
pub type LockResult<T> = Result<T, PoisonError<T>>;
pub type TryLockResult<T> = Result<T, TryLockError<T>>;

pub struct PoisonError<T> {
    guard: T,
}

impl<T> PoisonError<T> {
    pub fn new(guard: T) -> Self {
        Self { guard }
    }

    pub fn into_inner(self) -> T {
        self.guard
    }
}

impl<T> fmt::Debug for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoisonError").finish_non_exhaustive()
    }
}

impl<T> fmt::Display for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("lock poisoned")
    }
}

pub enum TryLockError<T> {
    Poisoned(PoisonError<T>),
    WouldBlock,
}

impl<T> fmt::Debug for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poisoned(_) => f.write_str("Poisoned(..)"),
            Self::WouldBlock => f.write_str("WouldBlock"),
        }
    }
}

pub trait Lock<T: ?Sized> {
    fn lock(&self) -> LockResult<MutexGuard<'_, T>>;
    fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>>;
}

pub struct SpinMutex<T: ?Sized> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized> {
    lock: &'a SpinMutex<T>,
    _not_send: PhantomData<*mut ()>,
}

impl<T> SpinMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn into_inner(self) -> LockResult<T> {
        Ok(self.data.into_inner())
    }
}

impl<T: ?Sized> SpinMutex<T> {
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        Ok(self.acquire())
    }

    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| MutexGuard {
                lock: self,
                _not_send: PhantomData,
            })
            .map_err(|_| TryLockError::WouldBlock)
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        Ok(self.data.get_mut())
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    fn acquire(&self) -> MutexGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop();
            }
        }
        MutexGuard {
            lock: self,
            _not_send: PhantomData,
        }
    }
}

impl<T: ?Sized> Lock<T> for SpinMutex<T> {
    fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        SpinMutex::lock(self)
    }

    fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        SpinMutex::try_lock(self)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_lock() {
            Ok(guard) => f.debug_struct("SpinMutex").field("data", &&*guard).finish(),
            Err(TryLockError::WouldBlock) => f
                .debug_struct("SpinMutex")
                .field("data", &"<locked>")
                .finish(),
            Err(TryLockError::Poisoned(_)) => f
                .debug_struct("SpinMutex")
                .field("data", &"<poisoned>")
                .finish(),
        }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

unsafe impl<T: ?Sized + Send> Send for SpinMutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for SpinMutex<T> {}
