use std::{alloc::Layout, ffi::c_void, ptr::NonNull};

use global::{attrs::ParameterAttr, derive_ctor::ctor};
use stdlib_header::CoreTypeId;

use crate::{
    stdlib::CoreTypeIdConstExt,
    type_system::{
        get_traits::{GetAssemblyRef, GetTypeVars},
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
};

use super::Method;

#[derive(ctor, Clone)]
#[ctor(pub const new)]
pub struct Parameter {
    pub(crate) ty: MaybeUnloadedTypeHandle,
    pub(crate) attr: ParameterAttr,
}

impl Parameter {
    pub const fn with_core_type(ty: CoreTypeId) -> Self {
        Self::new(ty.static_type_ref().into(), global::attr!(parameter {}))
    }
    pub const fn with_core_type_attr(ty: CoreTypeId, attr: ParameterAttr) -> Self {
        Self::new(ty.static_type_ref().into(), attr)
    }
}

impl Parameter {
    pub fn get_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        method: &Method<T>,
    ) -> NonGenericTypeHandle {
        match &self.ty {
            MaybeUnloadedTypeHandle::Loaded(ty) => ty.get_non_generic_with_method(method),
            MaybeUnloadedTypeHandle::Unloaded(_) => {
                let ty = unsafe {
                    self.ty
                        .load(
                            method
                                .mt
                                .unwrap()
                                .as_ref()
                                .ty_ref()
                                .__get_assembly_ref()
                                .manager_ref(),
                        )
                        .unwrap()
                };
                // Hacking
                unsafe {
                    NonNull::from_ref(self).as_mut().ty = MaybeUnloadedTypeHandle::Loaded(ty);
                }

                ty.get_non_generic_with_method(method)
            }
        }
    }
    pub fn get_layout<T: GetTypeVars + GetAssemblyRef>(&self, method: &Method<T>) -> Layout {
        if self.attr.is_by_ref() {
            return Layout::new::<*mut c_void>();
        }
        self.get_type(method).val_layout()
    }
    pub fn get_libffi_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        method: &Method<T>,
    ) -> libffi::middle::Type {
        if self.attr.is_by_ref() {
            return libffi::middle::Type::pointer();
        }
        self.get_type(method).val_libffi_type()
    }
}
