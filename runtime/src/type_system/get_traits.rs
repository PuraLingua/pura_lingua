mod _sealed {
    use crate::type_system::{class::Class, r#struct::Struct};

    pub trait TypeSealed {}

    macro default_impl($($T:ty)*) {$(
        impl TypeSealed for $T {}
    )*}

    default_impl! {
        Class
        Struct
    }
}

use std::{alloc::Layout, ptr::NonNull};

use super::{
    assembly::Assembly,
    class::Class,
    method::Method,
    method_table::MethodTable,
    r#struct::Struct,
    type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle, NonGenericTypeHandleKind},
};

use _sealed::*;

pub trait GetAssemblyRef: TypeSealed {
    fn __get_assembly_ref(&self) -> &Assembly;
}

pub trait GetTypeVars {
    fn __get_type_vars(&self) -> &Option<Box<[MaybeUnloadedTypeHandle]>>;
}

pub trait GetFields: TypeSealed {
    type Field = super::field::Field;

    fn __get_fields(&self) -> &Vec<Self::Field>;
    fn __get_fields_mut(&mut self) -> &mut Vec<Self::Field>;
}

pub trait GetParent: TypeSealed {
    fn __get_parent(&self) -> Option<NonNull<Self>>;
}

pub trait GetMethodTableRef: TypeSealed + Sized {
    fn __get_method_table_ref(&self) -> &MethodTable<Self>;
}

pub const trait GetValLayout {
    fn __get_val_layout(&self) -> Layout;
}

pub const trait GetValLibffiType {
    fn __get_val_libffi_type(&self) -> libffi::middle::Type;
}

pub const trait GetStaticConstructorId {
    fn __get_static_constructor_id(&self) -> u32;
}

pub const trait GetGeneric: Sized {
    fn __get_generic(&self) -> Option<NonNull<Self>>;
}

pub const trait GetNonGenericTypeHandleKind {
    fn __get_non_generic_type_handle_kind(&self) -> NonGenericTypeHandleKind;
}

macro get_assembly_ref_default_impl($($T:ty)*) {$(
	impl GetAssemblyRef for $T {
		fn __get_assembly_ref(&self) -> &Assembly {
			self.assembly_ref()
		}
	}
)*}

macro get_type_vars_default_impl($($T:ty)*) {$(
	impl GetTypeVars for $T {
		fn __get_type_vars(&self) -> &Option<Box<[MaybeUnloadedTypeHandle]>> {
			self.type_vars()
		}
	}
)*}

macro get_fields_default_impl($($T:ty)*) {$(
	impl GetFields for $T {
		type Field = super::field::Field;

		fn __get_fields(&self) -> &Vec<Self::Field> {
			self.fields()
		}
		fn __get_fields_mut(&mut self) -> &mut Vec<Self::Field> {
			self.fields_mut()
		}
	}
)*}

macro get_method_table_ref_default_impl($($T:ty)*) {$(
	impl GetMethodTableRef for $T {
		fn __get_method_table_ref(&self) -> &MethodTable<Self> {
			self.method_table_ref()
		}
	}
)*}

macro type_default_impls($($T:ty)*) {$(
	get_assembly_ref_default_impl!($T);
	get_type_vars_default_impl!($T);
	get_fields_default_impl!($T);
	get_method_table_ref_default_impl!($T);
)*}

type_default_impls! {
    Class
    Struct
}

impl<T> GetTypeVars for Method<T> {
    fn __get_type_vars(&self) -> &Option<Box<[MaybeUnloadedTypeHandle]>> {
        self.type_vars()
    }
}

impl GetParent for Class {
    fn __get_parent(&self) -> Option<NonNull<Self>> {
        *self.parent()
    }
}

impl GetParent for Struct {
    /// Structs do not have parents
    fn __get_parent(&self) -> Option<NonNull<Self>> {
        None
    }
}

impl const GetValLayout for Class {
    fn __get_val_layout(&self) -> Layout {
        self.val_layout()
    }
}

impl GetValLayout for Struct {
    fn __get_val_layout(&self) -> Layout {
        self.val_layout()
    }
}

impl GetValLibffiType for Class {
    fn __get_val_libffi_type(&self) -> libffi::middle::Type {
        self.val_libffi_type()
    }
}

impl GetValLibffiType for Struct {
    fn __get_val_libffi_type(&self) -> libffi::middle::Type {
        self.val_libffi_type()
    }
}

impl const GetStaticConstructorId for Class {
    fn __get_static_constructor_id(&self) -> u32 {
        *self.sctor()
    }
}

impl const GetStaticConstructorId for Struct {
    fn __get_static_constructor_id(&self) -> u32 {
        *self.sctor()
    }
}

impl const GetNonGenericTypeHandleKind for Class {
    fn __get_non_generic_type_handle_kind(&self) -> NonGenericTypeHandleKind {
        NonGenericTypeHandleKind::Class
    }
}

impl const GetNonGenericTypeHandleKind for Struct {
    fn __get_non_generic_type_handle_kind(&self) -> NonGenericTypeHandleKind {
        NonGenericTypeHandleKind::Struct
    }
}

impl const GetGeneric for Class {
    fn __get_generic(&self) -> Option<NonNull<Self>> {
        *self.generic()
    }
}

impl const GetGeneric for Struct {
    fn __get_generic(&self) -> Option<NonNull<Self>> {
        *self.generic()
    }
}
