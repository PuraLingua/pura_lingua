use std::{ffi::c_void, ptr::NonNull};

use global::instruction::{IRegisterAddr, Instruction_New};

use crate::{
    type_system::{
        cached_type_reference::GenericCachedTypeReference,
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        method::{
            Method, MethodRef,
            default_entry_point::{Termination, call_frame, load_register_failed},
        },
        type_handle::{MethodGenericResolver, NonGenericTypeHandle},
    },
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub(super) fn eval<T: GetAssemblyRef + GetTypeVars, TRegisterAddr: IRegisterAddr>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<c_void>,
    #[allow(unused)] pc: &mut usize,
    #[allow(unused)] caught_exception: Option<ManagedReference<Class>>,
    ins: &Instruction_New<GenericCachedTypeReference, MethodRef, TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    match ins {
        Instruction_New::NewObject {
            ty,
            ctor_name,
            args,
            output,
        } => {
            let args = args
                .iter()
                .map(|x| {
                    call_frame(cpu)
                        .get(*x)
                        .unwrap()
                        .ptr
                        .cast::<c_void>()
                        .as_ptr()
                })
                .collect::<Vec<_>>();

            let Some(class) = ty
                .get_with_generic_resolver(
                    method
                        .require_method_table_ref()
                        .ty_ref()
                        .__get_assembly_ref()
                        .manager_ref(),
                    MethodGenericResolver::new(method),
                )
                .and_then(|x| x.get_non_generic_with_method(method))
                .and_then(|x| match x {
                    NonGenericTypeHandle::Class(c) => Some(c),
                    _ => None,
                })
            else {
                return Some(Err(Termination::LoadTypeHandleFailed(
                    ty.to_maybe_unloaded_handle(),
                )));
            };

            match cpu.new_object(class, ctor_name, &args) {
                Some(obj) => {
                    if !call_frame(cpu).write_typed(*output, obj) {
                        load_register_failed!(*output);
                    }
                }
                None => {
                    return Some(Err(Termination::NewObjectFailed));
                }
            }
        }
        Instruction_New::NewArray {
            element_type,
            len,
            output,
        } => {
            let Some(element_th) = element_type
                .get_with_generic_resolver(
                    cpu.vm_ref().assembly_manager(),
                    MethodGenericResolver::new(method),
                )
                .and_then(|th| th.get_non_generic_with_method(method))
            else {
                return Some(Err(Termination::LoadTypeHandleFailed(
                    element_type.to_maybe_unloaded_handle(),
                )));
            };
            let arr = match element_th {
                NonGenericTypeHandle::Class(ty) => unsafe {
                    ManagedReference::alloc_array(cpu, ty.as_ref().method_table, (*len) as usize)
                },
                NonGenericTypeHandle::Struct(ty) => unsafe {
                    ManagedReference::alloc_array(cpu, ty.as_ref().method_table, (*len) as usize)
                },
                NonGenericTypeHandle::Interface(ty) => unsafe {
                    ManagedReference::alloc_array(cpu, ty.as_ref().method_table, (*len) as usize)
                },
            };
            if !call_frame(cpu).write_typed(*output, arr) {
                load_register_failed!(*output);
            }
        }
        Instruction_New::NewDynamicArray {
            element_type,
            len_addr,
            output,
        } => {
            let Some(&len) = call_frame(cpu).get_typed::<usize, _>(*len_addr) else {
                load_register_failed!(*len_addr);
            };
            let Some(element_th) = element_type
                .get_with_generic_resolver(
                    cpu.vm_ref().assembly_manager(),
                    MethodGenericResolver::new(method),
                )
                .and_then(|th| th.get_non_generic_with_method(method))
            else {
                return Some(Err(Termination::LoadTypeHandleFailed(
                    element_type.to_maybe_unloaded_handle(),
                )));
            };
            let arr = match element_th {
                NonGenericTypeHandle::Class(ty) => unsafe {
                    ManagedReference::alloc_array(cpu, ty.as_ref().method_table, len)
                },
                NonGenericTypeHandle::Struct(ty) => unsafe {
                    ManagedReference::alloc_array(cpu, ty.as_ref().method_table, len)
                },
                NonGenericTypeHandle::Interface(ty) => unsafe {
                    ManagedReference::alloc_array(cpu, ty.as_ref().method_table, len)
                },
            };
            if !call_frame(cpu).write_typed(*output, arr) {
                load_register_failed!(*output);
            }
        }
    }

    Some(Ok(()))
}
