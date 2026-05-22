use std::{ptr::NonNull, sync::RwLock};

use crate::{type_system::class::Class, value::managed_reference::ManagedReference};

#[allow(dead_code)]
pub struct ReflectionInfoContainer<T> {
    pub(crate) data: NonNull<T>,

    cache: RwLock<ManagedReference<Class>>,
}

impl<T> ReflectionInfoContainer<T> {
    pub const fn new(data: NonNull<T>) -> Self {
        Self {
            data,
            cache: RwLock::new(ManagedReference::null()),
        }
    }
}
