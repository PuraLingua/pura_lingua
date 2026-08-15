use std::{
    collections::HashMap,
    num::NonZero,
    ptr::NonNull,
    sync::{Arc, LazyLock, nonpoison::RwLock},
};

use crate::{
    type_system::{class::Class, get_traits::GetAssemblyRef},
    value::managed_reference::ManagedReference,
    virtual_machine::VirtualMachine,
};

pub trait IReflect: Sized {
    fn __get_reflect_container(&self) -> Option<&ReflectionInfoContainer<Self>>;
    fn __reflect_update(&self);
    fn __get_reflect_value(&self) -> ManagedReference<Class>;
}

#[allow(dead_code)]
pub struct ReflectionInfoContainer<T> {
    pub(crate) data: NonNull<T>,

    vm_getter: for<'a> fn(&'a T) -> &'a VirtualMachine,
    factor: Arc<dyn Fn(&VirtualMachine, NonNull<T>) -> ManagedReference<Class>>,
    cache: RwLock<ManagedReference<Class>>,
}

impl<T> ReflectionInfoContainer<T> {
    pub const fn new(
        data: NonNull<T>,
        vm_getter: for<'a> fn(&'a T) -> &'a VirtualMachine,
        factor: Arc<dyn Fn(&VirtualMachine, NonNull<T>) -> ManagedReference<Class>>,
    ) -> Self {
        Self {
            data,

            vm_getter,
            factor,
            cache: RwLock::new(ManagedReference::null()),
        }
    }

    /// [`crate::type_system::assembly::Assembly`] can use this too.
    pub const fn with_assembly_gettable(
        data: NonNull<T>,
        factor: Arc<dyn Fn(&VirtualMachine, NonNull<T>) -> ManagedReference<Class>>,
    ) -> Self
    where
        T: GetAssemblyRef,
    {
        Self {
            data,

            vm_getter: |x| x.__get_assembly_ref().manager_ref().vm_ref(),
            factor,
            cache: RwLock::new(ManagedReference::null()),
        }
    }

    #[inline(always)]
    pub const fn data_ref<'a>(&self) -> &'a T {
        unsafe { self.data.as_ref() }
    }

    pub fn update(&self) {
        let vm = (self.vm_getter)(self.data_ref());
        self.cache.set((self.factor)(vm, self.data));
    }

    pub fn value(&self) -> ManagedReference<Class> {
        self.cache.get_cloned()
    }
}

pub struct ReflectionInfoCache {
    inner: LazyLock<RwLock<HashMap<NonZero<usize>, ManagedReference<Class>>>>,
}

impl ReflectionInfoCache {
    pub const fn new() -> Self {
        Self {
            inner: LazyLock::new(|| RwLock::new(HashMap::new())),
        }
    }

    /// Null if not found
    #[inline(always)]
    pub fn get<T>(&self, ptr: NonNull<T>) -> ManagedReference<Class> {
        self.inner
            .read()
            .get(&ptr.addr())
            .copied()
            .unwrap_or(ManagedReference::null())
    }

    #[inline(always)]
    pub fn get_or<T>(
        &self,
        ptr: NonNull<T>,
        creator: impl FnOnce() -> ManagedReference<Class>,
    ) -> ManagedReference<Class> {
        *self.inner.write().entry(ptr.addr()).or_insert_with(creator)
    }
}
