use std::{alloc::Layout, fmt::Debug, num::NonZero, ptr::NonNull};

use derive_more::Display;
use global::{UnwrapEnum, non_purus_call_configuration::NonPurusCallType};

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
#[cfg(test)]
mod tests;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Display)]
pub enum NonGenericTypeHandleKind {
    Class = TypeHandleKind::Class as _,
    Struct = TypeHandleKind::Struct as _,
    Interface = TypeHandleKind::Interface as _,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FlattenedNonGenericTypeHandle {
    ptr: NonNull<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FlattenedTypeHandle {
    ptr: NonNull<u8>,
}

macro impl_flatten {
    (
        enum NonGeneric {
            $(
                #[is($is:ident)]
                #[unwrap($unwrap:ident)]
                #[unwrap_unchecked($unwrap_unchecked:ident)]
                #[try_into($try_into:ident)]
                $Variant:ident($VariantTy:ty)
            ),* $(,)?
        }
    ) => {
        impl FlattenedNonGenericTypeHandle {
            $(
                #[allow(non_snake_case)]
                pub fn $Variant(ptr: NonNull<$VariantTy>) -> Self {
                    const _: NonGenericTypeHandleKind = NonGenericTypeHandleKind::$Variant;
                    #[inline(always)]
                    const fn map_addr(addr: NonZero<usize>) -> NonZero<usize> {
                        TypeHandleKind::$Variant.tag(addr)
                    }
                    Self {
                        ptr: ptr.map_addr(map_addr).cast(),
                    }
                }

                pub fn $is(self) -> bool {
                    TypeHandleKind::get_tag(self.ptr.addr().get()) == TypeHandleKind::$Variant
                }
                pub fn $unwrap(self) -> NonNull<$VariantTy> {
                    if self.$is() {
                        unsafe { self.$unwrap_unchecked() }
                    } else {
                        const MSG: &str = concat!("The handle does not contain ", stringify!($Variant));
                        core::panicking::panic(MSG)
                    }
                }
                pub unsafe fn $unwrap_unchecked(self) -> NonNull<$VariantTy> {
                    self.ptr
                        .map_addr(|x| unsafe {
                            NonZero::new_unchecked(TypeHandleKind::untag(x.get()))
                        })
                        .cast()
                }
                pub fn $try_into(self) -> Option<NonNull<$VariantTy>> {
                    if self.$is() {
                        Some(unsafe { self.$unwrap_unchecked() })
                    } else {
                        None
                    }
                }
            )*
        }
    },
    (
        enum {
            $(
                #[is($is:ident)]
                #[unwrap($unwrap:ident)]
                #[unwrap_unchecked($unwrap_unchecked:ident)]
                #[try_into($try_into:ident)]
                $Variant:ident($VariantTy:ty)
            ),* $(,)?
        }
    ) => {
        impl FlattenedTypeHandle {
            $(
                #[allow(non_snake_case)]
                pub fn $Variant(ptr: NonNull<$VariantTy>) -> Self {
                    #[inline(always)]
                    const fn map_addr(addr: NonZero<usize>) -> NonZero<usize> {
                        TypeHandleKind::$Variant.tag(addr)
                    }
                    Self {
                        ptr: ptr.map_addr(map_addr).cast(),
                    }
                }

                pub fn $is(self) -> bool {
                    TypeHandleKind::get_tag(self.ptr.addr().get()) == TypeHandleKind::$Variant
                }
                pub fn $unwrap(self) -> NonNull<$VariantTy> {
                    if self.$is() {
                        unsafe { self.$unwrap_unchecked() }
                    } else {
                        const MSG: &str = concat!("The handle does not contain ", stringify!($Variant));
                        core::panicking::panic(MSG)
                    }
                }
                pub unsafe fn $unwrap_unchecked(self) -> NonNull<$VariantTy> {
                    self.ptr
                        .map_addr(|x| unsafe {
                            NonZero::new_unchecked(TypeHandleKind::untag(x.get()))
                        })
                        .cast()
                }
                pub fn $try_into(self) -> Option<NonNull<$VariantTy>> {
                    if self.$is() {
                        Some(unsafe { self.$unwrap_unchecked() })
                    } else {
                        None
                    }
                }
            )*
        }
    }
}

impl_flatten! {
    enum NonGeneric {
        #[is(is_class)]
        #[unwrap(unwrap_class)]
        #[unwrap_unchecked(unwrap_class_unchecked)]
        #[try_into(try_into_class)]
        Class(Class),
        #[is(is_struct)]
        #[unwrap(unwrap_struct)]
        #[unwrap_unchecked(unwrap_struct_unchecked)]
        #[try_into(try_into_struct)]
        Struct(Struct),
        #[is(is_interface)]
        #[unwrap(unwrap_interface)]
        #[unwrap_unchecked(unwrap_interface_unchecked)]
        #[try_into(try_into_interface)]
        Interface(Interface),
    }
}

impl FlattenedNonGenericTypeHandle {
    pub fn new(unflattened: NonGenericTypeHandle) -> Self {
        match unflattened {
            NonGenericTypeHandle::Class(ptr) => Self::Class(ptr),
            NonGenericTypeHandle::Struct(ptr) => Self::Struct(ptr),
            NonGenericTypeHandle::Interface(ptr) => Self::Interface(ptr),
        }
    }
}

impl_flatten! {
    enum {
        #[is(is_class)]
        #[unwrap(unwrap_class)]
        #[unwrap_unchecked(unwrap_class_unchecked)]
        #[try_into(try_into_class)]
        Class(Class),
        #[is(is_struct)]
        #[unwrap(unwrap_struct)]
        #[unwrap_unchecked(unwrap_struct_unchecked)]
        #[try_into(try_into_struct)]
        Struct(Struct),
        #[is(is_interface)]
        #[unwrap(unwrap_interface)]
        #[unwrap_unchecked(unwrap_interface_unchecked)]
        #[try_into(try_into_interface)]
        Interface(Interface),
    }
}

impl FlattenedTypeHandle {
    #[allow(non_snake_case)]
    pub fn MethodGeneric(i: u32) -> Self {
        Self {
            ptr: NonNull::without_provenance(
                TypeHandleKind::MethodGeneric
                    .tag_maybe_zero((i as usize) << (usize::BITS - u32::BITS)),
            ),
        }
    }
    pub fn is_method_generic(self) -> bool {
        TypeHandleKind::get_tag(self.ptr.addr().get()) == TypeHandleKind::MethodGeneric
    }
    pub fn unwrap_method_generic(self) -> u32 {
        if self.is_method_generic() {
            unsafe { self.unwrap_method_generic_unchecked() }
        } else {
            const MSG: &str = "The handle does not contain MethodGeneric";
            core::panicking::panic(MSG)
        }
    }
    pub unsafe fn unwrap_method_generic_unchecked(self) -> u32 {
        (self.ptr.addr().get() >> (usize::BITS - u32::BITS)) as u32
    }
    pub fn try_into_method_generic(self) -> Option<u32> {
        if self.is_method_generic() {
            Some(unsafe { self.unwrap_method_generic_unchecked() })
        } else {
            None
        }
    }

    #[allow(non_snake_case)]
    pub fn TypeGeneric(i: u32) -> Self {
        Self {
            ptr: NonNull::without_provenance(
                TypeHandleKind::TypeGeneric
                    .tag_maybe_zero((i as usize) << (usize::BITS - u32::BITS)),
            ),
        }
    }
    pub fn is_type_generic(self) -> bool {
        TypeHandleKind::get_tag(self.ptr.addr().get()) == TypeHandleKind::TypeGeneric
    }
    pub fn unwrap_type_generic(self) -> u32 {
        if self.is_type_generic() {
            unsafe { self.unwrap_method_generic_unchecked() }
        } else {
            const MSG: &str = "The handle does not contain TypeGeneric";
            core::panicking::panic(MSG)
        }
    }
    pub unsafe fn unwrap_type_generic_unchecked(self) -> u32 {
        (self.ptr.addr().get() >> (usize::BITS - u32::BITS)) as u32
    }
    pub fn try_into_type_generic(self) -> Option<u32> {
        if self.is_type_generic() {
            Some(unsafe { self.unwrap_type_generic_unchecked() })
        } else {
            None
        }
    }
}

impl FlattenedTypeHandle {
    pub fn new(unflattened: TypeHandle) -> Self {
        match unflattened {
            TypeHandle::Class(ptr) => Self::Class(ptr),
            TypeHandle::Struct(ptr) => Self::Struct(ptr),
            TypeHandle::Interface(ptr) => Self::Interface(ptr),
            TypeHandle::MethodGeneric(i) => Self::MethodGeneric(i),
            TypeHandle::TypeGeneric(i) => Self::TypeGeneric(i),
        }
    }
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

    pub fn name(&self) -> &widestring::Utf16Str {
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
#[derive_const(PartialEq, Eq)]
pub enum TypeHandleKind {
    Class = 1,
    Struct,
    Interface,
    MethodGeneric,
    TypeGeneric,
}

impl TypeHandleKind {
    pub const MAX: usize = std::mem::variant_count::<Self>();

    #[inline(always)]
    const fn tag(self, addr: NonZero<usize>) -> NonZero<usize> {
        addr | self as u8 as usize
    }
    #[inline(always)]
    const fn tag_maybe_zero(self, addr: usize) -> NonZero<usize> {
        unsafe { NonZero::new_unchecked(addr | self as u8 as usize) }
    }
    #[inline(always)]
    const fn untag(addr: usize) -> usize {
        addr & (usize::MAX << Self::MAX.bit_width())
    }
    #[inline(always)]
    const fn get_tag(addr: usize) -> Self {
        unsafe {
            std::mem::transmute(
                (addr & (usize::MAX >> (usize::BITS - Self::MAX.bit_width()))) as u8,
            )
        }
    }
}

const _: () = {
    assert!(
        align_of::<Class>() == align_of::<Struct>()
            && align_of::<Struct>() == align_of::<Interface>()
    );

    assert!(TypeHandleKind::MAX < align_of::<Class>());
};

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
    #[inline(always)]
    fn resolve_type_generic(&self, _g_index: u32) -> Option<TypeHandle> {
        None
    }

    #[inline(always)]
    fn resolve_method_generic(&self, _g_index: u32) -> Option<TypeHandle> {
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
