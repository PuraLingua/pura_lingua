use std::ptr::NonNull;

use either::Either;
use global::traits::IUnwrap;
use stdlib_header::CoreTypeRef;

use crate::{
    memory::OwnedPtr,
    stdlib::CoreTypeIdConstExt,
    type_system::{class::Class, interface::Interface, r#struct::Struct, type_ref::TypeRef},
};

use super::{MaybeUnloadedTypeHandle, NonGenericTypeHandle, TypeHandle};

const _: () = {
    macro easy_from_ptr($Source:ident -> $Target:ty) {
        impl const From<NonNull<$Source>> for $Target {
            fn from(value: NonNull<$Source>) -> Self {
                Self::$Source(value)
            }
        }

        impl const From<OwnedPtr<$Source>> for $Target {
            fn from(value: OwnedPtr<$Source>) -> Self {
                Self::$Source(value.as_non_null_ptr())
            }
        }
    }
    easy_from_ptr!(Class -> NonGenericTypeHandle);
    easy_from_ptr!(Struct -> NonGenericTypeHandle);
    easy_from_ptr!(Interface -> NonGenericTypeHandle);
};

impl From<CoreTypeRef> for MaybeUnloadedTypeHandle {
    fn from(value: CoreTypeRef) -> Self {
        match value {
            CoreTypeRef::Core(core_type_id) => core_type_id.static_type_ref().into(),
            CoreTypeRef::WithGeneric(main, generics) => Self::Unloaded(TypeRef::Specific {
                assembly_and_index: Either::Right(Box::new(main.static_type_ref().into())),
                types: generics.into_iter().map(Self::from).collect(),
            }),
            CoreTypeRef::MethodGeneric(g) => Self::Loaded(TypeHandle::MethodGeneric(g)),
            CoreTypeRef::TypeGeneric(g) => Self::Loaded(TypeHandle::TypeGeneric(g)),
        }
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
            NonGenericTypeHandle::Interface(ty) => Self::Interface(ty),
        }
    }
}

const _: () = {
    macro easy_unwrap($Source:ident from $Target:ty) {
        impl const IUnwrap<NonNull<$Source>> for $Target {
            fn _unwrap(self) -> NonNull<$Source> {
                match self {
                    <$Target>::$Source(x) => x,
                    _ => panic!("Call _unwrap with wrong type"),
                }
            }
        }

        impl<'a> const IUnwrap<&'a NonNull<$Source>> for &'a $Target {
            fn _unwrap(self) -> &'a NonNull<$Source> {
                match self {
                    <$Target>::$Source(x) => x,
                    _ => panic!("Call _unwrap with wrong type"),
                }
            }
        }
    }
    easy_unwrap!(Class from TypeHandle);
    easy_unwrap!(Struct from TypeHandle);
    easy_unwrap!(Interface from TypeHandle);

    easy_unwrap!(Class from NonGenericTypeHandle);
    easy_unwrap!(Struct from NonGenericTypeHandle);
    easy_unwrap!(Interface from NonGenericTypeHandle);
};
