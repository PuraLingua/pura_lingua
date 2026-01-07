use std::{alloc::Layout, fmt::Debug, pin::Pin, ptr::NonNull};

use global::{UnwrapEnum, traits::IUnwrap};

use crate::{
    stdlib::CoreTypeId,
    type_system::{
        assembly_manager::AssemblyManager, class::Class, r#struct::Struct, type_ref::TypeRef,
    },
};

use super::{
    get_traits::{GetAssemblyRef, GetTypeVars},
    method::Method,
};

mod convert;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum NonGenericTypeHandleKind {
    Class = TypeHandleKind::Class as _,
    Struct = TypeHandleKind::Struct as _,
}

#[repr(u8)]
#[derive(Clone, Copy, UnwrapEnum)]
#[unwrap_enum(owned)]
pub enum NonGenericTypeHandle {
    Class(NonNull<Class>) = NonGenericTypeHandleKind::Class as _,
    Struct(NonNull<Struct>) = NonGenericTypeHandleKind::Struct as _,
}

impl NonGenericTypeHandle {
    pub fn get_core_type_id(&self) -> Option<crate::stdlib::CoreTypeId> {
        match self {
            Self::Class(ty) => unsafe { ty.as_ref().method_table_ref().get_core_type_id() },
            Self::Struct(ty) => unsafe { ty.as_ref().method_table_ref().get_core_type_id() },
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

    pub const fn from_type_handle(th: TypeHandle) -> Option<NonGenericTypeHandle> {
        match th {
            TypeHandle::Class(ty) => Some(Self::Class(ty)),
            TypeHandle::Struct(ty) => Some(Self::Struct(ty)),
            TypeHandle::Generic(_) => None,
        }
    }

    pub fn val_layout(&self) -> Layout {
        match self {
            Self::Class(ty) => unsafe { ty.as_ref().val_layout() },
            Self::Struct(ty) => unsafe { ty.as_ref().val_layout() },
        }
    }

    pub fn val_libffi_type(&self) -> libffi::middle::Type {
        if self.is_certain_core_type(CoreTypeId::System_Void) {
            return libffi::middle::Type::void();
        }
        match self {
            Self::Class(ty) => unsafe { ty.as_ref().val_libffi_type() },
            Self::Struct(ty) => unsafe { ty.as_ref().val_libffi_type() },
        }
    }

    pub fn instantiate(&self, type_vars: &[MaybeUnloadedTypeHandle]) -> Self {
        unsafe {
            match self {
                Self::Class(ty) => Self::Class(ty.as_ref().instantiate(type_vars)),
                Self::Struct(ty) => Self::Struct(ty.as_ref().instantiate(type_vars)),
            }
        }
    }
}

impl NonGenericTypeHandle {
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
    Generic,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TypeHandle {
    Class(NonNull<Class>) = TypeHandleKind::Class as _,
    Struct(NonNull<Struct>) = TypeHandleKind::Struct as _,
    Generic(u32) = TypeHandleKind::Generic as _,
}

impl Debug for TypeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class(cl) => write!(f, "Class {}", unsafe { cl.as_ref().name() }),
            Self::Struct(s) => write!(f, "Struct {}", unsafe { s.as_ref().name() }),

            Self::Generic(g) => write!(f, "Generic {g}"),
        }
    }
}

impl PartialEq for TypeHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Class(p), Self::Class(po)) => p.addr() == po.addr(),
            (Self::Struct(p), Self::Struct(po)) => p.addr() == po.addr(),
            (Self::Generic(i1), Self::Generic(i2)) => i1.eq(i2),
            _ => false,
        }
    }
}

pub(crate) fn type_generic_resolver<T: GetTypeVars + GetAssemblyRef>(
    g_index: u32,
    ty: &T,
) -> TypeHandle {
    ty.__get_type_vars()
        .as_ref()
        .unwrap()
        .get(g_index as usize)
        .and_then(|x| x.load(ty.__get_assembly_ref().manager_ref()))
        .unwrap()
}

pub(crate) fn method_generic_resolver<T: GetTypeVars + GetAssemblyRef>(
    g_index: u32,
    method: &Method<T>,
) -> TypeHandle {
    let ty = method.require_method_table_ref().ty_ref();
    let ty_type_vars = ty.__get_type_vars();
    let ty_type_var_len = ty_type_vars.as_ref().map(|x| x.len()).unwrap_or(0);
    if (g_index as usize) >= ty_type_var_len {
        method
            .__get_type_vars()
            .as_ref()
            .unwrap()
            .get((g_index as usize) - ty_type_var_len)
            .and_then(|x| x.load(ty.__get_assembly_ref().manager_ref()))
    } else {
        // Safety: It's been checked
        unsafe {
            ty_type_vars
                .as_ref()
                .unwrap()
                .get_unchecked(g_index as usize)
                .load(ty.__get_assembly_ref().manager_ref())
        }
    }
    .unwrap()
}

impl TypeHandle {
    pub fn as_non_generic(&self) -> Option<&NonGenericTypeHandle> {
        match self {
            Self::Generic(_) => None,
            _ => unsafe { Some(&*(self as *const Self as *const NonGenericTypeHandle)) },
        }
    }

    pub const fn get_int_tag(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const Self as *const u8) }
    }

    pub fn get_non_generic_with_generic_resolver<F: Fn(u32) -> TypeHandle>(
        &self,
        f: F,
    ) -> NonGenericTypeHandle {
        match self.as_non_generic() {
            Some(x) => *x,
            None => {
                let Self::Generic(g_index) = self else {
                    unreachable!()
                };
                f(*g_index).get_non_generic_with_generic_resolver(f)
            }
        }
    }

    pub fn get_non_generic_with_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        ty: &T,
    ) -> NonGenericTypeHandle {
        self.get_non_generic_with_generic_resolver(|g_index| type_generic_resolver(g_index, ty))
    }

    pub fn get_non_generic_with_method<T: GetTypeVars + GetAssemblyRef>(
        &self,
        method: &Method<T>,
    ) -> NonGenericTypeHandle {
        self.get_non_generic_with_generic_resolver(|g_index| {
            method_generic_resolver(g_index, method)
        })
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

    pub fn val_layout_with_generic_resolver<F: Fn(u32) -> TypeHandle>(&self, f: F) -> Layout {
        self.get_non_generic_with_generic_resolver(f).val_layout()
    }

    pub fn val_layout_with_type<T: GetTypeVars + GetAssemblyRef>(&self, ty: &T) -> Layout {
        self.val_layout_with_generic_resolver(|g_index| type_generic_resolver(g_index, ty))
    }

    pub fn val_layout_with_method<T: GetTypeVars + GetAssemblyRef>(
        &self,
        method: &Method<T>,
    ) -> Layout {
        self.val_layout_with_generic_resolver(|g_index| method_generic_resolver(g_index, method))
    }
}

#[derive(Clone, Debug)]
pub enum MaybeUnloadedTypeHandle {
    Loaded(TypeHandle),
    Unloaded(TypeRef),
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
    pub fn load(&self, assembly_manager: &AssemblyManager) -> Option<TypeHandle> {
        match self {
            Self::Loaded(th) => Some(*th),
            Self::Unloaded(r) => match r {
                TypeRef::Index { assembly, ind } => {
                    let assembly = assembly_manager.get_assembly_by_name(assembly).unwrap()?;
                    assembly
                        .get_type_handle(*ind)
                        .unwrap()
                        .map(|x| *x)
                        .map(TypeHandle::from)
                }
                TypeRef::Specific {
                    assembly,
                    ind,
                    types,
                } => {
                    let assembly = assembly_manager.get_assembly_by_name(assembly).unwrap()?;
                    let ty = assembly.get_type_handle(*ind).unwrap()?;
                    Some(ty.instantiate(types).into())
                }
            },
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
    fn _assert()
    where
        And<
            And<
                ConstAssert<{ size_of::<TypeHandle>() >= size_of::<NonGenericTypeHandle>() }>,
                ConstAssert<{ test_align() }>,
            >,
            ConstAssert<{ test_tags() }>,
        >: SuccessAssert,
    {
    }
};
