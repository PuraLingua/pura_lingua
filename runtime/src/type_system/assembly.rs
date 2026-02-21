use std::{
    ptr::NonNull,
    sync::{MappedRwLockReadGuard, PoisonError, RwLock, RwLockReadGuard},
};

use global::traits::IUnwrap;

use crate::type_system::assembly_manager::AssemblyManager;

use super::{class::Class, r#struct::Struct, type_handle::NonGenericTypeHandle};

pub struct Assembly {
    pub(crate) manager: NonNull<AssemblyManager>,
    pub(crate) name: Box<str>,
    pub(crate) types: RwLock<Vec<NonGenericTypeHandle>>,

    pub(crate) is_core: bool,
}

impl Assembly {
    /// The NonNull passed to f is always valid to be cast to &Self
    pub fn new_for_adding<F: FnOnce(NonNull<Self>) -> Vec<NonGenericTypeHandle>>(
        name: String,
        is_core: bool,
        f: F,
    ) -> Box<Self> {
        let mut this = Box::new(Self {
            manager: NonNull::dangling(),
            name: name.into_boxed_str(),
            types: RwLock::new(Vec::new()),
            is_core,
        });

        let types = f(NonNull::from_ref(&*this));
        this.types = RwLock::new(types);

        this
    }
    pub fn new(manager: &AssemblyManager, name: String, is_core: bool) -> Self {
        Self {
            manager: NonNull::from_ref(manager),
            name: name.into_boxed_str(),
            types: RwLock::new(Vec::new()),
            is_core,
        }
    }

    #[inline]
    pub fn add_type<T>(&self, ty: NonNull<T>) -> u32
    where
        NonGenericTypeHandle: From<NonNull<T>>,
    {
        self.add_type_handle(NonGenericTypeHandle::from(ty))
    }

    pub fn add_type_handle(&self, ty: NonGenericTypeHandle) -> u32 {
        let mut types = self.types.write().unwrap();
        let index = types.len();
        types.push(ty);

        index as _
    }
}

impl Assembly {
    pub const fn manager_ref(&self) -> &AssemblyManager {
        unsafe { self.manager.as_ref() }
    }
}

#[allow(clippy::type_complexity)]
impl Assembly {
    /// More convenient sometimes but may panic
    pub fn get_type<T>(
        &self,
        index: u32,
    ) -> Result<
        Option<MappedRwLockReadGuard<'_, T>>,
        PoisonError<RwLockReadGuard<'_, Vec<NonGenericTypeHandle>>>,
    >
    where
        for<'a> &'a NonGenericTypeHandle: IUnwrap<&'a T>,
    {
        self.get_type_handle(index)
            .map(move |x| x.map(|x| MappedRwLockReadGuard::map(x, |x| x._unwrap())))
    }
    #[inline(always)]
    pub fn get_class(
        &self,
        index: u32,
    ) -> Result<
        Option<MappedRwLockReadGuard<'_, NonNull<Class>>>,
        PoisonError<RwLockReadGuard<'_, Vec<NonGenericTypeHandle>>>,
    > {
        self.get_type(index)
    }
    #[inline(always)]
    pub fn get_struct(
        &self,
        index: u32,
    ) -> Result<
        Option<MappedRwLockReadGuard<'_, NonNull<Struct>>>,
        PoisonError<RwLockReadGuard<'_, Vec<NonGenericTypeHandle>>>,
    > {
        self.get_type(index)
    }
    pub fn get_type_handle<'a>(
        &'a self,
        index: u32,
    ) -> Result<
        Option<MappedRwLockReadGuard<'a, NonGenericTypeHandle>>,
        PoisonError<RwLockReadGuard<'a, Vec<NonGenericTypeHandle>>>,
    > {
        self.types
            .read()
            .map(|x| RwLockReadGuard::filter_map(x, |x| x.get(index as usize)).ok())
    }
}
