use std::{ffi::c_void, ptr::NonNull};

use global::instruction::{IRegisterAddr, Instruction_Load, LoadContent};

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
    #[allow(unused)] caught_exception: Option<ManagedReference<Class>>,
    ins: &Instruction_Load<String, MaybeUnloadedTypeHandle, u32, TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    let register_addr = &ins.addr;
    match &ins.content {
        LoadContent::True => {
            if !call_frame(cpu).write_typed(ins.addr, true) {
                load_register_failed!(ins.addr);
            }
        }
        LoadContent::False => {
            if !call_frame(cpu).write_typed(*register_addr, false) {
                load_register_failed!(*register_addr);
            }
        }

        LoadContent::U8(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }
        LoadContent::U16(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }
        LoadContent::U32(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }
        LoadContent::U64(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }

        LoadContent::I8(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }
        LoadContent::I16(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }
        LoadContent::I32(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }
        LoadContent::I64(val) => {
            if !call_frame(cpu).write_typed(*register_addr, *val) {
                load_register_failed!(*register_addr);
            }
        }

        LoadContent::This => {
            let Some(local_var) = call_frame(cpu).get(*register_addr) else {
                load_register_failed!(*register_addr);
            };
            debug_assert!(local_var.layout.size() >= size_of::<*const ()>());
            let Some(this) = this else {
                return Some(Err(Termination::NullReference(
                    core::panic::Location::caller(),
                )));
            };
            unsafe {
                local_var.ptr.copy_from(this.cast(), size_of::<*const ()>());
            }
        }

        LoadContent::String(val) => {
            let val_obj = ManagedReference::new_string(cpu, val);
            if !call_frame(cpu).write_typed(*register_addr, val_obj) {
                load_register_failed!(*register_addr);
            }
        }

        LoadContent::TypeValueSize(ty) => {
            let Some(ty) = ty
                .load(cpu.vm_ref().assembly_manager())
                .and_then(|ty| ty.get_non_generic_with_method(method))
            else {
                return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
            };
            let size = ty.val_layout().size();
            if !call_frame(cpu).write_typed(*register_addr, size) {
                load_register_failed!(*register_addr);
            }
        }

        LoadContent::NonPurusCallConfiguration(val) => {
            let data = cpu.marshal_non_purus_configuration(val);
            if !call_frame(cpu).write_typed(*register_addr, data) {
                load_register_failed!(*register_addr);
            }
        }

        LoadContent::Arg(arg_index) => {
            let arg_index = (*arg_index) as usize;
            let Some(arg) = args.get(arg_index) else {
                return Some(Err(Termination::LoadArgFailed(arg_index as u64)));
            };

            match NonNull::new(*arg) {
                None => {
                    call_frame(cpu).zero_register(*register_addr);
                }
                Some(p) => {
                    let Some(out_var) = call_frame(cpu).get(*register_addr) else {
                        load_register_failed!(*register_addr);
                    };
                    if let Some(param) = method.args.get(arg_index)
                        && param.attr.is_by_ref()
                    {
                        out_var.write_typed(p);
                    } else {
                        unsafe { out_var.copy_all_from(p.cast()) }
                    }
                }
            }
        }
        LoadContent::ArgValue(arg) => {
            let Some(arg) = args.get((*arg) as usize) else {
                return Some(Err(Termination::LoadArgFailed(*arg)));
            };

            match NonNull::new(*arg) {
                None => {
                    call_frame(cpu).zero_register(*register_addr);
                }
                Some(p) => {
                    let Some(out_var) = call_frame(cpu).get(*register_addr) else {
                        load_register_failed!(*register_addr);
                    };
                    unsafe { out_var.copy_all_from(p.cast()) }
                }
            }
        }

        LoadContent::Static { ty, field } => {
            let Some(ty) = ty
                .load(cpu.vm_ref().assembly_manager())
                .map(|x| x.get_non_generic_with_method(method))
                .flatten()
            else {
                return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
            };
            let Some((f_ptr, f_layout)) = cpu.vm_ref().get_static_field(ty, *field) else {
                return Some(Err(Termination::LoadFieldFailed(*field)));
            };
            let Some(out_var) = call_frame(cpu).get(*register_addr) else {
                load_register_failed!(*register_addr);
            };
            unsafe {
                out_var.copy_from(f_ptr.cast(), f_layout.size());
            }
        }
        LoadContent::Field { container, field } => {
            let Some(container) = call_frame(cpu).get(*container) else {
                load_register_failed!(*container);
            };
            let Some(register_var) = call_frame(cpu).get(*register_addr) else {
                load_register_failed!(*register_addr);
            };
            match container.ty {
                NonGenericTypeHandle::Class(_) => {
                    let Some((field_ptr, field_layout)) = container
                        .as_ref_typed::<ManagedReference<Class>>()
                        .const_access::<FieldAccessor<Class>>()
                        .field(*field, Default::default())
                    else {
                        return Some(Err(Termination::LoadFieldFailed(*field)));
                    };
                    debug_assert!(register_var.layout.size() >= field_layout.size());
                    unsafe {
                        register_var.copy_from(field_ptr, field_layout.size());
                    }
                }
                NonGenericTypeHandle::Struct(s) => {
                    let Some(field_info) = unsafe { s.as_ref() }.method_table_ref().field_mem_info(
                        *field,
                        Default::default(),
                        Default::default(),
                    ) else {
                        return Some(Err(Termination::LoadFieldFailed(*field)));
                    };
                    debug_assert!(register_var.layout.size() >= field_info.layout.size());
                    unsafe {
                        register_var.copy_from(
                            container.ptr.byte_add(field_info.offset),
                            field_info.layout.size(),
                        );
                    }
                }
                NonGenericTypeHandle::Interface(_) => unreachable!(),
            }
        }

        LoadContent::CaughtException => {}
    }

    Some(Ok(()))
}
