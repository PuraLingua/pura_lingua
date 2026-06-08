use std::{ptr::NonNull, sync::nonpoison::RwLock};

use global::{UnwrapEnum, traits::IUnwrap};

use crate::{
    memory::OwnedPtr,
    type_system::{assembly_manager::AssemblyManager, interface::Interface},
};

use super::{class::Class, r#struct::Struct, type_handle::NonGenericTypeHandle};

#[derive(UnwrapEnum)]
#[unwrap_enum(owned)]
pub enum TypeContainer {
    Class(Box<Class>),
    Struct(Box<Struct>),
    Interface(Box<Interface>),
}

impl TypeContainer {
    pub const fn handle(&self) -> NonGenericTypeHandle {
        match self {
            TypeContainer::Class(ty) => NonGenericTypeHandle::Class(NonNull::from_ref(&**ty)),
            TypeContainer::Struct(ty) => NonGenericTypeHandle::Struct(NonNull::from_ref(&**ty)),
            TypeContainer::Interface(ty) => {
                NonGenericTypeHandle::Interface(NonNull::from_ref(&**ty))
            }
        }
    }
    pub fn name(&self) -> &widestring::Utf16Str {
        match self {
            TypeContainer::Class(ty) => &**ty.name(),
            TypeContainer::Struct(ty) => &**ty.name(),
            TypeContainer::Interface(ty) => &**ty.name(),
        }
    }
}

impl From<OwnedPtr<Class>> for TypeContainer {
    fn from(value: OwnedPtr<Class>) -> Self {
        Self::Class(unsafe { Box::from_non_null(value.as_non_null_ptr()) })
    }
}

impl From<OwnedPtr<Struct>> for TypeContainer {
    fn from(value: OwnedPtr<Struct>) -> Self {
        Self::Struct(unsafe { Box::from_non_null(value.as_non_null_ptr()) })
    }
}

impl From<OwnedPtr<Interface>> for TypeContainer {
    fn from(value: OwnedPtr<Interface>) -> Self {
        Self::Interface(unsafe { Box::from_non_null(value.as_non_null_ptr()) })
    }
}

pub struct Assembly {
    pub(crate) manager: NonNull<AssemblyManager>,
    pub(crate) name: Box<widestring::Utf16Str>,
    pub(crate) types: RwLock<Vec<TypeContainer>>,

    pub(crate) is_core: bool,
}

impl Assembly {
    /// The NonNull passed to f is always valid to be cast to &Self
    pub fn new_for_adding<F: FnOnce(NonNull<Self>) -> Vec<TypeContainer>>(
        name: widestring::Utf16String,
        is_core: bool,
        f: F,
    ) -> Box<Self> {
        let mut this = Box::new(Self {
            manager: NonNull::dangling(),
            name: name.into_boxed_utfstr(),
            types: RwLock::new(Vec::new()),
            is_core,
        });

        let types = f(NonNull::from_ref(&*this));
        this.types = RwLock::new(types);

        this
    }
    pub fn new(manager: &AssemblyManager, name: widestring::Utf16String, is_core: bool) -> Self {
        Self {
            manager: NonNull::from_ref(manager),
            name: name.into_boxed_utfstr(),
            types: RwLock::new(Vec::new()),
            is_core,
        }
    }

    #[inline]
    pub fn add_type<T>(&self, ty: OwnedPtr<T>) -> u32
    where
        TypeContainer: From<OwnedPtr<T>>,
    {
        self.add_type_handle(TypeContainer::from(ty))
    }

    pub fn add_type_handle(&self, ty: TypeContainer) -> u32 {
        let mut types = self.types.write();
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
    pub fn get_type<T>(&self, index: u32) -> Option<T>
    where
        NonGenericTypeHandle: IUnwrap<T>,
    {
        self.get_type_handle(index).map(IUnwrap::_unwrap)
    }
    pub fn get_type_handle<'a>(&'a self, index: u32) -> Option<NonGenericTypeHandle> {
        self.types
            .read()
            .get(index as usize)
            .map(TypeContainer::handle)
    }

    pub fn find_type_handle<'a>(
        &'a self,
        name: impl AsRef<widestring::Utf16Str>,
    ) -> Option<NonGenericTypeHandle> {
        let name = name.as_ref();
        self.types
            .read()
            .iter()
            .find(|x| x.name() == name)
            .map(TypeContainer::handle)
    }
    /// More convenient sometimes but may panic
    pub fn find_type<T>(&self, name: impl AsRef<widestring::Utf16Str>) -> Option<T>
    where
        NonGenericTypeHandle: IUnwrap<T>,
    {
        self.find_type_handle(name).map(IUnwrap::_unwrap)
    }
}

macro gen_gets($(
    fn $name:ident() -> $Ty:ty;
)*) {$(
    #[inline(always)]
    pub fn $name(
        &self,
        index: u32,
    ) -> Option<NonNull<$Ty>> {
        self.get_type(index)
    }
)*}

macro gen_finds($(
    fn $name:ident() -> $Ty:ty;
)*) {$(
    #[inline(always)]
    pub fn $name(
        &self,
        name: impl AsRef<widestring::Utf16Str>,
    ) -> Option<NonNull<$Ty>> {
        self.find_type(name)
    }
)*}

impl Assembly {
    gen_gets!(
        fn get_class() -> Class;
        fn get_struct() -> Struct;
        fn get_interface() -> Interface;
    );
    gen_finds!(
        fn find_class() -> Class;
        fn find_struct() -> Struct;
        fn find_interface() -> Interface;
    );
}
