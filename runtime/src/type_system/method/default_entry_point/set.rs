use std::{ffi::c_void, ptr::NonNull};

use global::instruction::{IRegisterAddr, Instruction_Set};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        method::{
            Method,
            default_entry_point::{Termination, call_frame, load_register_failed},
        },
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

pub(super) fn eval<T: Sized + GetAssemblyRef + GetTypeVars, TRegisterAddr: IRegisterAddr>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    ins: &Instruction_Set<MaybeUnloadedTypeHandle, u32, TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    match ins {
        Instruction_Set::Common {
            val,
            container,
            field,
        } => {
            let Some(container) = call_frame(cpu).get(*container) else {
                load_register_failed!(*container);
            };
            let Some(val) = call_frame(cpu).get(*val) else {
                load_register_failed!(*val);
            };
            match container.ty {
                NonGenericTypeHandle::Class(_) => {
                    let Some((out, out_layout)) = container
                        .as_ref_typed::<ManagedReference<Class>>()
                        .const_access::<FieldAccessor<Class>>()
                        .field(*field, Default::default())
                    else {
                        return Some(Err(Termination::LoadFieldFailed(*field)));
                    };
                    debug_assert!(val.layout.size() >= out_layout.size());
                    unsafe {
                        val.copy_to(out, out_layout.size());
                    }
                }
                NonGenericTypeHandle::Struct(s) => {
                    let mt_ref = unsafe { s.as_ref().method_table_ref() };
                    let Some(field_info) =
                        mt_ref.field_mem_info(*field, Default::default(), Default::default())
                    else {
                        return Some(Err(Termination::LoadFieldFailed(*field)));
                    };
                    debug_assert!(val.layout.size() >= field_info.layout.size());
                    unsafe {
                        val.copy_to(
                            container.ptr.byte_add(field_info.offset),
                            field_info.layout.size(),
                        );
                    }
                }
            }
        }
        Instruction_Set::This { val, field } => {
            let Some(val_var) = call_frame(cpu).get(*val) else {
                load_register_failed!(*val);
            };
            let Some(this) = this else {
                return Some(Err(Termination::NullReference(
                    core::panic::Location::caller(),
                )));
            };
            unsafe {
                let this = this.cast::<ManagedReference<Class>>().as_ref();
                let Some((f_ptr, f_layout)) = this
                    .const_access::<FieldAccessor<_>>()
                    .field(*field, Default::default())
                else {
                    return Some(Err(Termination::LoadFieldFailed(*field)));
                };
                val_var.copy_to(f_ptr.cast(), f_layout.size());
            }
        }
        Instruction_Set::Static { val, ty, field } => {
            let Some(ty) = ty
                .load(cpu.vm_ref().assembly_manager())
                .map(|x| x.get_non_generic_with_method(method))
            else {
                return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
            };
            let Some((f_ptr, f_layout)) = cpu.vm_ref().get_static_field(ty, *field) else {
                return Some(Err(Termination::LoadFieldFailed(*field)));
            };
            let Some(val_var) = call_frame(cpu).get(*val) else {
                load_register_failed!(*val);
            };
            unsafe {
                val_var.copy_to(f_ptr.cast(), f_layout.size());
            }
        }
    }

    Some(Ok(()))
}
