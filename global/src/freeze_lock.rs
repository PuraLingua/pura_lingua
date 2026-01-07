use std::{
    cell::UnsafeCell,
    sync::{RwLock, atomic::AtomicBool},
};

/// A type which allows mutation using a lock until
/// the value is frozen and can be accessed lock-free.
///
/// Unlike `RwLock`, it can be used to prevent mutation past a point.
#[derive(Default)]
#[allow(unused)]
pub struct FreezeLock<T> {
    data: UnsafeCell<T>,
    frozen: AtomicBool,

    /// This lock protects writes to the `data` and `frozen` fields.
    lock: RwLock<()>,
}
