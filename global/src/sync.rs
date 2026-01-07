use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockResult};

pub struct LoggedRwLock<T>(RwLock<T>);

impl<T> LoggedRwLock<T> {
    #[track_caller]
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        println!("Lock at {}", std::panic::Location::caller());
        self.0.read()
    }
    #[track_caller]
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        println!("Lock at {}", std::panic::Location::caller());
        self.0.write()
    }
    #[track_caller]
    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        println!("Lock at {}", std::panic::Location::caller());
        self.0.try_read()
    }
    #[track_caller]
    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, T>> {
        println!("Lock at {}", std::panic::Location::caller());
        self.0.try_write()
    }
}
