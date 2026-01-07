use std::{ffi::c_void, mem::offset_of, ptr::NonNull};

use global::{
    attrs::{CallConvention, MethodAttr, MethodImplementationFlags},
    dt_println,
    getset::Getters,
    instruction::Instruction,
};
use libffi::low::CodePtr;

use crate::{
    stdlib::CoreTypeId,
    type_system::{
        generics::GenericBounds, method_table::MethodTable, type_handle::MaybeUnloadedTypeHandle,
    },
    virtual_machine::cpu::CPU,
};

use super::{
    get_traits::{GetAssemblyRef, GetTypeVars},
    type_handle::NonGenericTypeHandle,
};

mod calling;
mod parameter;

pub use parameter::Parameter;

#[derive(Getters)]
#[getset(get = "pub")]
pub struct Method<T> {
    #[getset(skip)]
    mt: Option<NonNull<MethodTable<T>>>,
    generic: Option<NonNull<Self>>,

    name: Box<str>,
    attr: MethodAttr<MaybeUnloadedTypeHandle>,
    args: Vec<Parameter>,
    return_type: MaybeUnloadedTypeHandle,
    #[getset(skip)]
    call_convention: CallConvention,

    generic_instances: Vec<NonNull<Self>>,
    generic_bounds: Option<NonNull<[GenericBounds]>>,
    type_vars: Option<Box<[MaybeUnloadedTypeHandle]>>,

    instructions: Vec<Instruction<MaybeUnloadedTypeHandle, MethodRef, u32>>,
    entry_point: CodePtr,
}

mod display;

pub use display::MethodDisplayOptions;

pub(crate) mod default_entry_point;

impl<T> Method<T> {
    pub const fn require_method_table_ref(&self) -> &MethodTable<T> {
        unsafe { self.mt.unwrap().as_ref() }
    }
}

#[allow(clippy::too_many_arguments)]
impl<T> Method<T>
where
    T: GetTypeVars + GetAssemblyRef,
{
    pub fn new(
        mt: NonNull<MethodTable<T>>,

        name: String,
        attr: MethodAttr<MaybeUnloadedTypeHandle>,
        args: Vec<Parameter>,
        return_type: MaybeUnloadedTypeHandle,
        call_convention: CallConvention,

        generic_bounds: Option<Vec<GenericBounds>>,

        instructions: Vec<Instruction<MaybeUnloadedTypeHandle, MethodRef, u32>>,
    ) -> Self {
        Self {
            mt: Some(mt),
            generic: None,

            name: name.into_boxed_str(),
            attr,
            args,
            return_type,
            call_convention,

            generic_instances: Vec::new(),
            generic_bounds: generic_bounds
                .filter(|x| !x.is_empty())
                .map(|x| Box::into_non_null(x.into_boxed_slice())),
            type_vars: None,

            instructions,
            entry_point: CodePtr::from_ptr(default_entry_point::__default_entry_point::<T> as _),
        }
    }
}

#[allow(clippy::too_many_arguments)]
impl<T> Method<T> {
    pub fn native(
        mt: Option<NonNull<MethodTable<T>>>,

        name: String,
        attr: MethodAttr<MaybeUnloadedTypeHandle>,
        args: Vec<Parameter>,
        return_type: MaybeUnloadedTypeHandle,
        call_convention: CallConvention,

        generic_bounds: Option<Vec<GenericBounds>>,

        entry_point: *const c_void,
    ) -> Self {
        Self {
            mt,
            generic: None,

            name: name.into_boxed_str(),
            attr,
            args,
            return_type,
            call_convention,

            generic_instances: Vec::new(),
            generic_bounds: generic_bounds
                .filter(|x| !x.is_empty())
                .map(|x| Box::into_non_null(x.into_boxed_slice())),
            type_vars: None,

            instructions: Vec::new(),
            entry_point: CodePtr::from_ptr(entry_point),
        }
    }
    pub fn default_sctor(
        mt: Option<NonNull<MethodTable<T>>>,
        mut attr: MethodAttr<MaybeUnloadedTypeHandle>,
    ) -> Self {
        extern "system" fn sctor<T>(_: &CPU, _: &Method<T>) {
            dt_println!("Calling default sctor");
        }
        attr.impl_flags_mut()
            .insert(MethodImplementationFlags::Static);
        Self::native(
            mt,
            ".sctor".to_owned(),
            attr,
            vec![],
            CoreTypeId::System_Void.static_type_ref().into(),
            CallConvention::PlatformDefault,
            None,
            sctor::<T> as _,
        )
    }
}

impl<T> Method<T> {
    pub fn instantiate(&self, type_vars: &[MaybeUnloadedTypeHandle]) -> NonNull<Self> {
        for has_instantiated in self.generic_instances.iter() {
            if unsafe { has_instantiated.as_ref() }
                .type_vars
                .as_deref()
                .is_some_and(|x| x.eq(type_vars))
            {
                return *has_instantiated;
            }
        }

        let instantiated = Box::new(Self {
            mt: self.mt,
            generic: Some(NonNull::from_ref(self)),

            name: self.name.clone(),
            attr: self.attr.clone(),

            args: self.args.clone(),
            return_type: self.return_type.clone(),
            call_convention: self.call_convention,
            instructions: self.instructions.clone(),
            entry_point: self.entry_point,

            generic_instances: Vec::new(),
            generic_bounds: None,
            type_vars: Some(Box::clone_from_ref(type_vars)),
        });

        let instantiated = Box::into_non_null(instantiated);

        unsafe {
            NonNull::from_ref(self)
                .byte_add(offset_of!(Self, generic_instances))
                .cast::<Vec<NonNull<Self>>>()
                .as_mut()
                .push(instantiated);
        }

        instantiated
    }
}

impl<T> Method<T> {
    pub const fn call_convention(&self) -> CallConvention {
        match &self.call_convention {
            CallConvention::PlatformDefault => cfg_select! {
                all(windows, target_arch = "x86_64") => { CallConvention::Win64 }
                all(windows, target_arch = "x86") => { CallConvention::Stdcall }
            },

            x => *x,
        }
    }
    const fn libffi_call_convention(&self) -> libffi::middle::FfiAbi {
        match self.call_convention {
            CallConvention::PlatformDefault => libffi::middle::ffi_abi_FFI_DEFAULT_ABI,
            CallConvention::CDecl => cfg_select! {
                target_arch = "x86" => { libffi::middle::ffi_abi_FFI_MS_CDECL }
                _ => { libffi::raw::ffi_abi_FFI_WIN64 }
            },
            CallConvention::CDeclWithVararg => cfg_select! {
                target_arch = "x86" => { libffi::middle::ffi_abi_FFI_MS_CDECL }
                _ => { libffi::raw::ffi_abi_FFI_WIN64 }
            },
            CallConvention::Stdcall => cfg_select! {
                target_arch = "x86" => { libffi::middle::ffi_abi_FFI_STDCALL }
                _ => { libffi::raw::ffi_abi_FFI_WIN64 }
            },
            CallConvention::Fastcall => cfg_select! {
                target_arch = "x86" => { libffi::middle::ffi_abi_FFI_FASTCALL }
                _ => { libffi::raw::ffi_abi_FFI_DEFAULT_ABI }
            },
            CallConvention::Win64 => cfg_select! {
                all(windows, target_arch = "x86_64") => { libffi::raw::ffi_abi_FFI_WIN64 }
                _ => { libffi::raw::ffi_abi_FFI_DEFAULT_ABI }
            },
            CallConvention::SystemV => cfg_select! {
                /* cSpell:disable-next-line */
                all(unix, target_arch = "x86_64") => { libffi::raw::ffi_abi_FFI_GNUW64 }
                _ => { libffi::raw::ffi_abi_FFI_DEFAULT_ABI }
            },
        }
    }
}

impl<T: GetTypeVars + GetAssemblyRef> Method<T> {
    pub fn get_return_type(&self) -> NonGenericTypeHandle {
        match &self.return_type {
            MaybeUnloadedTypeHandle::Loaded(ty) => ty.get_non_generic_with_method(self),
            MaybeUnloadedTypeHandle::Unloaded(_) => {
                let ty = self
                    .return_type
                    .load(unsafe {
                        self.mt
                            .unwrap()
                            .as_ref()
                            .ty_ref()
                            .__get_assembly_ref()
                            .manager_ref()
                    })
                    .unwrap();
                // Hacking
                unsafe {
                    NonNull::from_ref(self).as_mut().return_type =
                        MaybeUnloadedTypeHandle::Loaded(ty);
                }

                ty.get_non_generic_with_method(self)
            }
        }
    }
    fn libffi_return_type(&self) -> libffi::middle::Type {
        if self
            .get_return_type()
            .is_certain_core_type(CoreTypeId::System_Void)
        {
            libffi::middle::Type::void()
        } else {
            self.get_return_type().val_libffi_type()
        }
    }
}

const _: () = {
    use global::assertions::*;
    fn _assert()
    where
        for<'a> LayoutEq<libffi::middle::Arg<'a>, *mut c_void>: SuccessAssert,
    {
    }
};

#[derive(Clone, Debug)]
pub enum MethodRef {
    Index(u32),
    Specific {
        index: u32,
        types: Vec<MaybeUnloadedTypeHandle>,
    },
}

#[cfg(test)]
mod tests;
