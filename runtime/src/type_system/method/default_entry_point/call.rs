use std::{alloc::Layout, ffi::c_void, ptr::NonNull};

use global::instruction::{IRegisterAddr, Instruction_Call};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        method::{
            Method, MethodRef,
            default_entry_point::{Termination, call_frame, load_register_failed},
        },
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub(super) fn eval<T: Sized + GetAssemblyRef + GetTypeVars, TRegisterAddr: IRegisterAddr>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    ins: &Instruction_Call<MaybeUnloadedTypeHandle, MethodRef, TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    match ins {
        Instruction_Call::InstanceCall {
            val,
            method,
            args,
            ret_at,
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

            let Some(val) = call_frame(cpu).read_typed::<ManagedReference<Class>, _>(*val) else {
                load_register_failed!(*val);
            };
            if val.is_null() {
                return Some(Err(Termination::NullReference(
                    core::panic::Location::caller(),
                )));
            }
            let mt = val.method_table_ref().unwrap();
            let Some(m) = mt.get_method_by_ref(method) else {
                return Some(Err(Termination::LoadMethodFailed(method.clone())));
            };

            let m_ref = unsafe { m.as_ref() };
            let actual_layout = m_ref.get_return_type().val_layout();
            let (ret_ptr, ret_layout) =
                m_ref.untyped_call(cpu, Some(NonNull::from_ref(&val).cast()), &args);

            if actual_layout != Layout::new::<()>() {
                let Some(out_var) = call_frame(cpu).get(*ret_at) else {
                    load_register_failed!(*ret_at);
                };
                unsafe {
                    out_var.copy_from(ret_ptr, actual_layout.size());
                }
            }
            unsafe {
                std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
            }
        }
        Instruction_Call::StaticCall {
            ty,
            method: m_target,
            args,
            ret_at,
        } => {
            let Some(ty) = ty
                .load(cpu.vm_ref().assembly_manager())
                .map(|x| x.get_non_generic_with_method(method))
            else {
                return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
            };

            let args = args
                .iter()
                .map(|x| {
                    call_frame(cpu)
                        .get(*x)
                        .unwrap()
                        .ptr()
                        .cast::<c_void>()
                        .as_ptr()
                })
                .collect::<Vec<_>>();

            macro apply($n:ident $b:block) {
                match ty {
                    NonGenericTypeHandle::Class($n) => $b,
                    NonGenericTypeHandle::Struct($n) => $b,
                }
            }

            let actual_layout;

            let (ret_ptr, ret_layout) = apply! {
                t {
                    let t_ref = unsafe { t.as_ref() };
                    let mt = t_ref.method_table_ref();
                    let m = mt.get_method_by_ref(m_target).unwrap();
                    let m_ref = unsafe { m.as_ref() };
                    actual_layout = m_ref.get_return_type().val_layout();

                    m_ref.untyped_call(cpu, None, &args)
                }
            };

            if actual_layout != Layout::new::<()>() {
                let Some(out_var) = call_frame(cpu).get(*ret_at) else {
                    load_register_failed!(*ret_at);
                };
                unsafe {
                    out_var.copy_from(ret_ptr, actual_layout.size());
                }
            }
            unsafe {
                std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
            }
        }
        Instruction_Call::StaticNonPurusCall {
            f_pointer,
            config,
            args,
            ret_at,
        } => {
            let Some(&f_pointer) = call_frame(cpu).get_typed::<*const u8, _>(*f_pointer) else {
                load_register_failed!(*f_pointer);
            };
            let args = args
                .iter()
                .map(|x| {
                    call_frame(cpu)
                        .get(*x)
                        .unwrap()
                        .ptr()
                        .cast::<c_void>()
                        .as_ptr()
                })
                .collect::<Vec<_>>();
            let (ret_ptr, ret_layout) = cpu.non_purus_call(config, f_pointer, args);
            let Some(ret_out) = call_frame(cpu).get(*ret_at) else {
                load_register_failed!(*ret_at);
            };
            let true_ret_layout = config.return_type.layout();
            unsafe {
                ret_out.copy_from(ret_ptr, true_ret_layout.size());
                std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
            }
        }
        Instruction_Call::DynamicNonPurusCall {
            f_pointer,
            config,
            args,
            ret_at,
        } => {
            let Some(f_pointer) = call_frame(cpu).read_typed::<*const u8, _>(*f_pointer) else {
                load_register_failed!(*f_pointer);
            };
            let Some(config) = call_frame(cpu).read_typed::<ManagedReference<Class>, _>(*config)
            else {
                load_register_failed!(*config);
            };
            let args = args
                .iter()
                .map(|x| {
                    call_frame(cpu)
                        .get(*x)
                        .unwrap()
                        .ptr()
                        .cast::<c_void>()
                        .as_ptr()
                })
                .collect::<Vec<_>>();
            let cfg = match cpu.unmarshal_non_purus_configuration(config) {
                Ok(x) => x,
                Err(err) => return Some(Err(Termination::UnmarshalFailed(err))),
            };
            let (ret_ptr, ret_layout) = cpu.non_purus_call(&cfg, f_pointer, args);
            let Some(ret_out) = call_frame(cpu).get(*ret_at) else {
                load_register_failed!(*ret_at);
            };
            let true_ret_layout = cfg.return_type.layout();
            unsafe {
                ret_out.copy_from(ret_ptr, true_ret_layout.size());
                std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
            }
        }
    }

    Some(Ok(()))
}
