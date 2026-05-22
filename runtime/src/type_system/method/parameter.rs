use std::{alloc::Layout, ffi::c_void};

use global::attrs::ParameterAttr;
use stdlib_header::CoreTypeId;

use crate::{
    stdlib::CoreTypeIdConstExt,
    type_system::{
        cached_type_reference::CachedTypeReference,
        get_traits::{GetAssemblyRef, GetTypeVars},
        type_handle::{MaybeUnloadedTypeHandle, MethodGenericResolver, NonGenericTypeHandle},
    },
};

use super::Method;

#[derive(Clone)]
pub struct Parameter {
    pub(crate) ty: CachedTypeReference,
    pub(crate) attr: ParameterAttr,
}

impl Parameter {
    pub fn new(ty: MaybeUnloadedTypeHandle, attr: ParameterAttr) -> Self {
        Self {
            ty: CachedTypeReference::new(ty),
            attr,
        }
    }
}

impl Parameter {
    pub fn with_core_type(ty: CoreTypeId) -> Self {
        Self::new(ty.static_type_ref().into(), global::attr!(parameter {}))
    }
    pub fn with_core_type_attr(ty: CoreTypeId, attr: ParameterAttr) -> Self {
        Self::new(ty.static_type_ref().into(), attr)
    }
}

impl Parameter {
    pub fn get_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        method: &Method<T>,
    ) -> NonGenericTypeHandle {
        self.ty
            .get_with_generic_resolver(
                unsafe {
                    method
                        .mt
                        .unwrap()
                        .as_ref()
                        .ty_ref()
                        .__get_assembly_ref()
                        .manager_ref()
                },
                MethodGenericResolver::new(method),
            )
            .unwrap()
            .get_non_generic_with_method(method)
            .unwrap()
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
