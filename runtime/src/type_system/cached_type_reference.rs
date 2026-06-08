use std::sync::nonpoison::RwLock;

use either::Either;

use crate::type_system::{
    assembly_manager::AssemblyManager,
    type_handle::{IGenericResolver, MaybeUnloadedTypeHandle, NonGenericTypeHandle, TypeHandle},
    type_ref::TypeRef,
};

#[derive(Clone, Debug)]
enum OriginTypeReference {
    Unloaded(TypeRef),
    NotInstantiated {
        handle: TypeHandle,
        type_vars: Vec<MaybeUnloadedTypeHandle>,
    },
    Already(TypeHandle),
}

impl OriginTypeReference {
    fn to_handle(&self) -> Option<TypeHandle> {
        match self {
            OriginTypeReference::Unloaded(_) => None,
            OriginTypeReference::NotInstantiated { .. } => None,
            OriginTypeReference::Already(type_handle) => Some(*type_handle),
        }
    }
    fn restore(&self) -> MaybeUnloadedTypeHandle {
        match self {
            OriginTypeReference::Unloaded(type_ref) => {
                MaybeUnloadedTypeHandle::Unloaded(type_ref.clone())
            }
            OriginTypeReference::NotInstantiated { handle, type_vars } => {
                MaybeUnloadedTypeHandle::Unloaded(TypeRef::Specific {
                    assembly_and_index: Either::Right(Box::new(MaybeUnloadedTypeHandle::Loaded(
                        *handle,
                    ))),
                    types: type_vars.clone(),
                })
            }
            OriginTypeReference::Already(type_handle) => {
                MaybeUnloadedTypeHandle::Loaded(*type_handle)
            }
        }
    }
    fn load_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        assembly_manager: &AssemblyManager,
        resolver: &TResolver,
    ) -> Option<TypeHandle> {
        match self {
            OriginTypeReference::Unloaded(type_ref) => {
                type_ref.load_with_generic_resolver(assembly_manager, resolver)
            }
            OriginTypeReference::NotInstantiated { handle, type_vars } => {
                let type_vars = type_vars
                    .iter()
                    .map(|x| {
                        x.load_with_generic_resolver(assembly_manager, resolver)
                            .and_then(|x| x.get_non_generic_with_generic_resolver(resolver))
                    })
                    .try_collect::<Vec<_>>()?;
                Some(
                    handle
                        .get_non_generic_with_generic_resolver(resolver)?
                        .instantiate(&type_vars)
                        .into(),
                )
            }
            OriginTypeReference::Already(handle) => Some(*handle),
        }
    }
    fn has_generic(&self) -> bool {
        fn has_generic(ty: &MaybeUnloadedTypeHandle) -> bool {
            match ty {
                MaybeUnloadedTypeHandle::Loaded(type_handle) => match type_handle {
                    TypeHandle::Class(_) | TypeHandle::Struct(_) | TypeHandle::Interface(_) => {
                        false
                    }
                    TypeHandle::MethodGeneric(_) | TypeHandle::TypeGeneric(_) => true,
                },
                MaybeUnloadedTypeHandle::Unloaded(type_ref) => match type_ref {
                    TypeRef::Index { .. } => false,
                    TypeRef::Specific {
                        assembly_and_index: _,
                        types,
                    } => types.iter().any(has_generic),
                },
            }
        }
        match self {
            OriginTypeReference::Unloaded(type_ref) => match type_ref {
                TypeRef::Index { .. } => false,
                TypeRef::Specific {
                    assembly_and_index: _,
                    types,
                } => types.iter().any(has_generic),
            },
            OriginTypeReference::NotInstantiated { handle, type_vars } => {
                has_generic(&MaybeUnloadedTypeHandle::Loaded(*handle))
                    || type_vars.iter().any(has_generic)
            }
            OriginTypeReference::Already(type_handle) => {
                has_generic(&MaybeUnloadedTypeHandle::Loaded(*type_handle))
            }
        }
    }
}

impl From<MaybeUnloadedTypeHandle> for OriginTypeReference {
    fn from(value: MaybeUnloadedTypeHandle) -> Self {
        match value {
            MaybeUnloadedTypeHandle::Loaded(type_handle) => Self::Already(type_handle),
            MaybeUnloadedTypeHandle::Unloaded(type_ref) => match type_ref {
                TypeRef::Index { .. } => Self::Unloaded(type_ref),
                TypeRef::Specific {
                    assembly_and_index,
                    types,
                } => match assembly_and_index {
                    Either::Left(assembly_and_index) => Self::Unloaded(TypeRef::Specific {
                        assembly_and_index: Either::Left(assembly_and_index),
                        types,
                    }),
                    Either::Right(handle) => match Box::into_inner(handle) {
                        MaybeUnloadedTypeHandle::Loaded(type_handle) => Self::NotInstantiated {
                            handle: type_handle,
                            type_vars: types,
                        },
                        MaybeUnloadedTypeHandle::Unloaded(generic) => {
                            Self::Unloaded(TypeRef::Specific {
                                assembly_and_index: Either::Right(Box::new(
                                    MaybeUnloadedTypeHandle::Unloaded(generic),
                                )),
                                types,
                            })
                        }
                    },
                },
            },
        }
    }
}

pub struct CachedTypeReference {
    inner: OriginTypeReference,
    cache: RwLock<Option<NonGenericTypeHandle>>,
}

impl std::fmt::Debug for CachedTypeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.cache.get_cloned() {
            None => match &self.inner {
                OriginTypeReference::Unloaded(type_ref) => <_ as std::fmt::Debug>::fmt(type_ref, f),
                OriginTypeReference::NotInstantiated { handle, type_vars } => {
                    write!(f, "{handle:?}{type_vars:?}")
                }
                OriginTypeReference::Already(type_handle) => {
                    <_ as std::fmt::Debug>::fmt(type_handle, f)
                }
            },
            Some(x) => <_ as std::fmt::Debug>::fmt(&x, f),
        }
    }
}

impl std::fmt::Display for CachedTypeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <_ as std::fmt::Display>::fmt(&self.to_maybe_unloaded_handle(), f)
    }
}

impl Clone for CachedTypeReference {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cache: RwLock::new(None),
        }
    }
}

impl<T> From<T> for CachedTypeReference
where
    MaybeUnloadedTypeHandle: From<T>,
{
    fn from(ty: T) -> Self {
        Self::new(ty.into())
    }
}

impl CachedTypeReference {
    pub fn new(ty: MaybeUnloadedTypeHandle) -> Self {
        Self {
            inner: ty.into(),
            cache: RwLock::new(None),
        }
    }

    pub fn to_maybe_unloaded_handle(&self) -> MaybeUnloadedTypeHandle {
        self.inner.restore()
    }
    pub fn to_handle(&self) -> Option<TypeHandle> {
        let cache = self.cache.read();
        if let Some(cache) = *cache {
            return Some(cache.into());
        }
        self.inner.to_handle()
    }
    pub fn assume_init(&self) -> TypeHandle {
        self.to_handle().unwrap()
    }
    pub fn get_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        assembly_manager: &AssemblyManager,
        resolver: &TResolver,
    ) -> Option<TypeHandle> {
        let mut cache = self.cache.write();

        if let Some(cache) = *cache {
            return Some(cache.into());
        }

        let result = self
            .inner
            .load_with_generic_resolver(assembly_manager, resolver)?;
        *cache = result.get_non_generic_with_generic_resolver(resolver);

        Some(result)
    }
}

pub struct GenericCachedTypeReference {
    inner: OriginTypeReference,
    has_generic: bool,
    cache: RwLock<Option<TypeHandle>>,
}

impl std::fmt::Debug for GenericCachedTypeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.cache.get_cloned() {
            None => match &self.inner {
                OriginTypeReference::Unloaded(type_ref) => <_ as std::fmt::Debug>::fmt(type_ref, f),
                OriginTypeReference::NotInstantiated { handle, type_vars } => {
                    write!(f, "{handle:?}{type_vars:?}")
                }
                OriginTypeReference::Already(type_handle) => {
                    <_ as std::fmt::Debug>::fmt(type_handle, f)
                }
            },
            Some(x) => <_ as std::fmt::Debug>::fmt(&x, f),
        }
    }
}

impl std::fmt::Display for GenericCachedTypeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <_ as std::fmt::Display>::fmt(&self.to_maybe_unloaded_handle(), f)
    }
}

impl Clone for GenericCachedTypeReference {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            has_generic: self.has_generic,
            cache: RwLock::new(None),
        }
    }
}

impl<T> From<T> for GenericCachedTypeReference
where
    MaybeUnloadedTypeHandle: From<T>,
{
    fn from(ty: T) -> Self {
        Self::new(ty.into())
    }
}

impl GenericCachedTypeReference {
    pub fn new(ty: MaybeUnloadedTypeHandle) -> Self {
        let inner: OriginTypeReference = ty.into();
        Self {
            has_generic: inner.has_generic(),
            inner,
            cache: RwLock::new(None),
        }
    }

    pub fn to_maybe_unloaded_handle(&self) -> MaybeUnloadedTypeHandle {
        self.inner.restore()
    }
    pub fn to_handle(&self) -> Option<TypeHandle> {
        let cache = self.cache.read();
        if let Some(cache) = *cache {
            return Some(cache);
        }
        self.inner.to_handle()
    }
    pub fn assume_init(&self) -> TypeHandle {
        self.to_handle().unwrap()
    }
    pub fn get_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        assembly_manager: &AssemblyManager,
        resolver: &TResolver,
    ) -> Option<TypeHandle> {
        let mut cache = self.cache.write();

        if let Some(cache) = *cache {
            return Some(cache);
        }

        let result = self
            .inner
            .load_with_generic_resolver(assembly_manager, resolver)?;
        if !self.has_generic {
            *cache = Some(result);
        }

        Some(result)
    }
}
