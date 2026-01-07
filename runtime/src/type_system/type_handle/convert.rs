use std::ptr::{NonNull, Unique};

use global::traits::IUnwrap;

use crate::type_system::{class::Class, r#struct::Struct, type_ref::TypeRef};

use super::{MaybeUnloadedTypeHandle, NonGenericTypeHandle, TypeHandle};

impl const From<NonNull<Class>> for NonGenericTypeHandle {
    fn from(value: NonNull<Class>) -> Self {
        Self::Class(value)
    }
}

impl const From<NonNull<Struct>> for NonGenericTypeHandle {
    fn from(value: NonNull<Struct>) -> Self {
        Self::Struct(value)
    }
}

impl const From<Unique<Class>> for NonGenericTypeHandle {
    fn from(value: Unique<Class>) -> Self {
        Self::Class(value.as_non_null_ptr())
    }
}

impl const From<Unique<Struct>> for NonGenericTypeHandle {
    fn from(value: Unique<Struct>) -> Self {
        Self::Struct(value.as_non_null_ptr())
    }
}

impl<T> const From<T> for MaybeUnloadedTypeHandle
where
    TypeHandle: [const] From<T>,
{
    fn from(value: T) -> Self {
        Self::Loaded(value.into())
    }
}

impl const From<TypeRef> for MaybeUnloadedTypeHandle {
    fn from(value: TypeRef) -> Self {
        Self::Unloaded(value)
    }
}

impl<T> const From<T> for TypeHandle
where
    NonGenericTypeHandle: [const] From<T>,
{
    default fn from(value: T) -> Self {
        Self::from(NonGenericTypeHandle::from(value))
    }
}

impl const From<NonGenericTypeHandle> for TypeHandle {
    fn from(value: NonGenericTypeHandle) -> Self {
        match value {
            NonGenericTypeHandle::Class(ty) => Self::Class(ty),
            NonGenericTypeHandle::Struct(ty) => Self::Struct(ty),
        }
    }
}

impl const IUnwrap<NonNull<Class>> for TypeHandle {
    fn _unwrap(self) -> NonNull<Class> {
        match self {
            Self::Class(c) => c,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}

impl const IUnwrap<NonNull<Struct>> for TypeHandle {
    fn _unwrap(self) -> NonNull<Struct> {
        match self {
            Self::Struct(s) => s,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}

impl<'a> const IUnwrap<&'a NonNull<Class>> for &'a TypeHandle {
    fn _unwrap(self) -> &'a NonNull<Class> {
        match self {
            TypeHandle::Class(c) => c,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}

impl<'a> const IUnwrap<&'a NonNull<Struct>> for &'a TypeHandle {
    fn _unwrap(self) -> &'a NonNull<Struct> {
        match self {
            TypeHandle::Struct(s) => s,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}

// NonGeneric
impl const IUnwrap<NonNull<Class>> for NonGenericTypeHandle {
    fn _unwrap(self) -> NonNull<Class> {
        match self {
            Self::Class(c) => c,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}

impl const IUnwrap<NonNull<Struct>> for NonGenericTypeHandle {
    fn _unwrap(self) -> NonNull<Struct> {
        match self {
            Self::Struct(s) => s,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}

impl<'a> const IUnwrap<&'a NonNull<Class>> for &'a NonGenericTypeHandle {
    fn _unwrap(self) -> &'a NonNull<Class> {
        match self {
            NonGenericTypeHandle::Class(c) => c,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}

impl<'a> const IUnwrap<&'a NonNull<Struct>> for &'a NonGenericTypeHandle {
    fn _unwrap(self) -> &'a NonNull<Struct> {
        match self {
            NonGenericTypeHandle::Struct(s) => s,
            _ => panic!("Call _unwrap with wrong type"),
        }
    }
}
