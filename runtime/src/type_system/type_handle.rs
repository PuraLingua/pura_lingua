use std::{alloc::Layout, fmt::Debug, ptr::NonNull};

use derive_more::Display;
use global::{UnwrapEnum, dt_println, non_purus_call_configuration::NonPurusCallType};

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdExt},
    type_system::{
        assembly_manager::AssemblyManager, class::Class, interface::Interface, r#struct::Struct,
        type_ref::TypeRef,
    },
};

use super::{
    get_traits::{GetAssemblyRef, GetTypeVars},
    method::Method,
};

mod convert;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Display)]
pub enum NonGenericTypeHandleKind {
    Class = TypeHandleKind::Class as _,
    Struct = TypeHandleKind::Struct as _,
    Interface = TypeHandleKind::Interface as _,
}

#[repr(u8)]
#[derive(Clone, Copy, UnwrapEnum, PartialEq, Eq)]
#[unwrap_enum(owned)]
pub enum NonGenericTypeHandle {
    Class(NonNull<Class>) = NonGenericTypeHandleKind::Class as _,
    Struct(NonNull<Struct>) = NonGenericTypeHandleKind::Struct as _,
    Interface(NonNull<Interface>) = NonGenericTypeHandleKind::Interface as _,
}

impl Debug for NonGenericTypeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class(cl) => write!(f, "Class {}", unsafe { cl.as_ref().name() }),
            Self::Struct(s) => write!(f, "Struct {}", unsafe { s.as_ref().name() }),
            Self::Interface(s) => write!(f, "Interface {}", unsafe { s.as_ref().name() }),
        }
    }
}

impl NonGenericTypeHandle {
    pub fn get_core_type_id(&self) -> Option<crate::stdlib::CoreTypeId> {
        match self {
            Self::Class(ty) => unsafe { ty.as_ref().method_table_ref().get_core_type_id() },
            Self::Struct(ty) => unsafe { ty.as_ref().method_table_ref().get_core_type_id() },
            Self::Interface(ty) => unsafe { ty.as_ref().method_table_ref().get_core_type_id() },
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Class(ty) => unsafe { ty.as_ref().name() },
            Self::Struct(ty) => unsafe { ty.as_ref().name() },
            Self::Interface(ty) => unsafe { ty.as_ref().name() },
        }
    }

    pub fn is_certain_core_type(&self, id: CoreTypeId) -> bool {
        self.get_core_type_id().is_some_and(|x| x == id)
    }

    pub fn is_pointer(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Self::Class(_) => true,
            _ => false,
        }
    }

    pub const fn is_managed_reference(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Self::Class(_) => true,
            _ => false,
        }
    }

    pub const fn from_type_handle(th: TypeHandle) -> Option<NonGenericTypeHandle> {
        match th {
            TypeHandle::Class(ty) => Some(Self::Class(ty)),
            TypeHandle::Struct(ty) => Some(Self::Struct(ty)),
            TypeHandle::Interface(ty) => Some(Self::Interface(ty)),
            TypeHandle::MethodGeneric(_) => None,
            TypeHandle::TypeGeneric(_) => None,
        }
    }

    pub fn val_layout(&self) -> Layout {
        match self {
            Self::Class(ty) => unsafe { ty.as_ref().val_layout() },
            Self::Struct(ty) => unsafe { ty.as_ref().val_layout() },
            Self::Interface(ty) => unsafe { ty.as_ref().val_layout() },
        }
    }

    pub fn val_libffi_type(&self) -> libffi::middle::Type {
        if self.is_certain_core_type(CoreTypeId::System_Void) {
            return libffi::middle::Type::void();
        }
        match self {
            Self::Class(ty) => unsafe { ty.as_ref().val_libffi_type() },
            Self::Struct(ty) => unsafe { ty.as_ref().val_libffi_type() },
            Self::Interface(_) => libffi::middle::Type::pointer(),
        }
    }

    pub fn non_purus_call_type(&self) -> NonPurusCallType {
        if let Some(core_type_id) = self.get_core_type_id()
            && let Some(gotten_ty) = core_type_id.non_purus_call_type()
        {
            return gotten_ty;
        }

        match self {
            Self::Class(_) => NonPurusCallType::Object,
            Self::Struct(ty) => unsafe { ty.as_ref().non_purus_call_type() },
            Self::Interface(_) => NonPurusCallType::Object,
        }
    }

    pub fn instantiate(&self, type_vars: &[NonGenericTypeHandle]) -> Self {
        unsafe {
            match self {
                Self::Class(ty) => Self::Class(ty.as_ref().instantiate(type_vars)),
                Self::Struct(ty) => Self::Struct(ty.as_ref().instantiate(type_vars)),
                Self::Interface(ty) => Self::Interface(ty.as_ref().instantiate(type_vars)),
            }
        }
    }
}

impl NonGenericTypeHandle {
    pub const fn get_tag(&self) -> NonGenericTypeHandleKind {
        unsafe { std::mem::transmute::<_, NonGenericTypeHandleKind>(self.get_int_tag()) }
    }
    pub const fn get_int_tag(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const Self as *const u8) }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TypeHandleKind {
    Class,
    Struct,
    Interface,
    MethodGeneric,
    TypeGeneric,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TypeHandle {
    Class(NonNull<Class>) = TypeHandleKind::Class as _,
    Struct(NonNull<Struct>) = TypeHandleKind::Struct as _,
    Interface(NonNull<Interface>) = TypeHandleKind::Interface as _,
    MethodGeneric(u32) = TypeHandleKind::MethodGeneric as _,
    TypeGeneric(u32) = TypeHandleKind::TypeGeneric as _,
}

impl Debug for TypeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class(cl) => write!(f, "Class {}", unsafe { cl.as_ref().name() }),
            Self::Struct(s) => write!(f, "Struct {}", unsafe { s.as_ref().name() }),
            Self::Interface(s) => write!(f, "Interface {}", unsafe { s.as_ref().name() }),

            Self::MethodGeneric(g) => write!(f, "!!{g}"),
            Self::TypeGeneric(g) => write!(f, "!{g}"),
        }
    }
}

impl PartialEq for TypeHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Class(p), Self::Class(po)) => p.addr() == po.addr(),
            (Self::Struct(p), Self::Struct(po)) => p.addr() == po.addr(),
            (Self::Interface(p), Self::Interface(po)) => p.addr() == po.addr(),
            (Self::MethodGeneric(i1), Self::MethodGeneric(i2)) => i1.eq(i2),
            (Self::TypeGeneric(i1), Self::TypeGeneric(i2)) => i1.eq(i2),
            _ => false,
        }
    }
}

pub trait IGenericResolver {
    fn resolve_type_generic(&self, g_index: u32) -> Option<TypeHandle>;
    fn resolve_method_generic(&self, g_index: u32) -> Option<TypeHandle>;
}

/// Return None for every generic
pub struct GenericUnresolvable;

impl IGenericResolver for GenericUnresolvable {
    fn resolve_type_generic(&self, _g_index: u32) -> Option<TypeHandle> {
        dt_println!("Unreachable code may be reached");
        None
    }

    fn resolve_method_generic(&self, _g_index: u32) -> Option<TypeHandle> {
        dt_println!("Unreachable code may be reached");
        None
    }
}

#[repr(transparent)]
pub struct TypeGenericResolver<T>(T);

impl<T> TypeGenericResolver<T> {
    pub fn new<'a>(ty: &'a T) -> &'a Self {
        unsafe { &*(ty as *const _ as *const Self) }
    }
}

impl<T: GetTypeVars + GetAssemblyRef> IGenericResolver for TypeGenericResolver<T> {
    fn resolve_type_generic(&self, g_index: u32) -> Option<TypeHandle> {
        self.0
            .__get_type_vars()
            .as_ref()
            .unwrap()
            .get(g_index as usize)
            .copied()
            .map(TypeHandle::from)
    }
    #[inline(always)]
    fn resolve_method_generic(&self, _: u32) -> Option<TypeHandle> {
        None
    }
}

#[repr(transparent)]
pub struct MethodGenericResolver<T>(Method<T>);

impl<T> MethodGenericResolver<T> {
    pub fn new<'a>(method: &'a Method<T>) -> &'a Self {
        unsafe { &*(method as *const _ as *const Self) }
    }
}

impl<T: GetTypeVars + GetAssemblyRef> IGenericResolver for MethodGenericResolver<T> {
    fn resolve_type_generic(&self, g_index: u32) -> Option<TypeHandle> {
        let ty = self.0.require_method_table_ref().ty_ref();
        ty.__get_type_vars()
            .as_ref()
            .unwrap()
            .get(g_index as usize)
            .copied()
            .map(TypeHandle::from)
    }

    fn resolve_method_generic(&self, g_index: u32) -> Option<TypeHandle> {
        self.0
            .__get_type_vars()
            .as_ref()
            .unwrap()
            .get(g_index as usize)
            .copied()
            .map(TypeHandle::from)
    }
}

impl TypeHandle {
    pub fn as_non_generic(&self) -> Option<&NonGenericTypeHandle> {
        match self {
            Self::MethodGeneric(_) => None,
            Self::TypeGeneric(_) => None,
            _ => unsafe { Some(&*(self as *const Self as *const NonGenericTypeHandle)) },
        }
    }

    pub fn into_non_generic(self) -> Option<NonGenericTypeHandle> {
        match self {
            Self::MethodGeneric(_) => None,
            Self::TypeGeneric(_) => None,
            x => unsafe { Some(std::mem::transmute::<Self, NonGenericTypeHandle>(x)) },
        }
    }

    pub const fn get_int_tag(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const Self as *const u8) }
    }

    pub fn get_non_generic_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        resolver: &TResolver,
    ) -> Option<NonGenericTypeHandle> {
        match self.as_non_generic() {
            Some(x) => Some(*x),
            None => match self {
                TypeHandle::MethodGeneric(g_index) => {
                    TResolver::resolve_method_generic(resolver, *g_index)
                        .and_then(|th| th.get_non_generic_with_generic_resolver(resolver))
                }
                TypeHandle::TypeGeneric(g_index) => {
                    TResolver::resolve_type_generic(resolver, *g_index)
                        .and_then(|th| th.get_non_generic_with_generic_resolver(resolver))
                }

                _ => unreachable!(),
            },
        }
    }

    pub fn get_non_generic_with_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        ty: &T,
    ) -> Option<NonGenericTypeHandle> {
        self.get_non_generic_with_generic_resolver(TypeGenericResolver::new(ty))
    }

    pub fn get_non_generic_with_method<T: GetTypeVars + GetAssemblyRef>(
        &self,
        method: &Method<T>,
    ) -> Option<NonGenericTypeHandle> {
        self.get_non_generic_with_generic_resolver(MethodGenericResolver::new(method))
    }

    pub fn val_layout(&self) -> Option<Layout> {
        self.as_non_generic().map(|x| x.val_layout())
    }

    /// # Safety
    /// `&self` must not matches [`TypeHandle::Generic`]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn val_layout_ignore_generic(&self) -> Layout {
        self.as_non_generic().unwrap_unchecked().val_layout()
    }

    pub fn val_layout_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        resolver: &TResolver,
    ) -> Layout {
        self.get_non_generic_with_generic_resolver(resolver)
            .unwrap()
            .val_layout()
    }

    pub fn val_layout_with_type<T: GetTypeVars + GetAssemblyRef>(&self, ty: &T) -> Layout {
        self.val_layout_with_generic_resolver(TypeGenericResolver::new(ty))
    }

    pub fn val_layout_with_method<T: GetTypeVars + GetAssemblyRef>(
        &self,
        method: &Method<T>,
    ) -> Layout {
        self.val_layout_with_generic_resolver(MethodGenericResolver::new(method))
    }
}

#[derive(Clone)]
pub enum MaybeUnloadedTypeHandle {
    Loaded(TypeHandle),
    Unloaded(TypeRef),
}

impl std::fmt::Debug for MaybeUnloadedTypeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Loaded(type_handle) => {
                format!("{type_handle:?}")
            }
            Self::Unloaded(type_ref) => {
                format!("Unloaded({type_ref:?})")
            }
        })
    }
}

impl std::fmt::Display for MaybeUnloadedTypeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Loaded(type_handle) => {
                format!("{type_handle:?}")
            }
            Self::Unloaded(type_ref) => {
                format!("Unloaded({type_ref:?})")
            }
        })
    }
}

impl PartialEq for MaybeUnloadedTypeHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Loaded(s), Self::Loaded(o)) => s.eq(o),
            (Self::Unloaded(s), Self::Unloaded(o)) => s.eq(o),
            _ => false,
        }
    }
}

impl MaybeUnloadedTypeHandle {
    pub fn assume_init(self) -> TypeHandle {
        const fn raise() -> ! {
            panic!("TypeHandle is not loaded")
        }
        match self {
            Self::Loaded(th) => th,
            _ => raise(),
        }
    }
    pub fn assume_init_ref(&self) -> &TypeHandle {
        const fn raise() -> ! {
            panic!("TypeHandle is not loaded")
        }
        match self {
            Self::Loaded(th) => th,
            _ => raise(),
        }
    }
    pub fn load_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        assembly_manager: &AssemblyManager,
        resolver: &TResolver,
    ) -> Option<TypeHandle> {
        match self {
            Self::Loaded(th) => Some(*th),
            Self::Unloaded(r) => r.load_with_generic_resolver(assembly_manager, resolver),
        }
    }
}

#[allow(warnings)]
const _: () = {
    use global::assertions::*;
    const fn test_align() -> bool {
        const fn is_aligned_to(this: *mut (), align: usize) -> bool {
            if !align.is_power_of_two() {
                panic!("is_aligned_to: align is not a power-of-two");
            }

            unsafe { std::mem::transmute::<_, usize>(this) & (align - 1) == 0 }
        }
        is_aligned_to(
            NonNull::<TypeHandle>::dangling().as_ptr().cast(),
            align_of::<NonGenericTypeHandle>(),
        )
    }
    const fn test_tags() -> bool {
        const CLASS: TypeHandle = TypeHandle::Class(NonNull::dangling());
        const STRUCT: TypeHandle = TypeHandle::Struct(NonNull::dangling());

        const N_CLASS: NonGenericTypeHandle = NonGenericTypeHandle::Class(NonNull::dangling());
        const N_STRUCT: NonGenericTypeHandle = NonGenericTypeHandle::Struct(NonNull::dangling());

        (CLASS.get_int_tag() == N_CLASS.get_int_tag())
            && (STRUCT.get_int_tag() == N_STRUCT.get_int_tag())
    }

    assert!(size_of::<TypeHandle>() >= size_of::<NonGenericTypeHandle>());
    assert!(test_align());
    assert!(test_tags());
};
