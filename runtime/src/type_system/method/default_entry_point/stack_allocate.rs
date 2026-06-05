use std::{ffi::c_void, ptr::NonNull};

use global::instruction::{IRegisterAddr, Instruction_StackAllocate};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        method::{
            Method,
            default_entry_point::{Termination, call_frame, load_register_failed},
        },
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
    ins: &Instruction_StackAllocate<TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    let (out, size, align, should_zero) = match ins {
        Instruction_StackAllocate::Dynamic { out, size, align } => {
            let Some(out) = call_frame(cpu).get_mut_typed::<NonNull<u8>, _>(*out) else {
                load_register_failed!(*out);
            };
            let Some(size) = call_frame(cpu).read_typed::<usize, _>(*size) else {
                load_register_failed!(*size);
            };
            let Some(align) = call_frame(cpu).read_typed::<usize, _>(*align) else {
                load_register_failed!(*align);
            };
            (out, size, align, false)
        }
        Instruction_StackAllocate::DynamicZeroed { out, size, align } => {
            let Some(out) = call_frame(cpu).get_mut_typed::<NonNull<u8>, _>(*out) else {
                load_register_failed!(*out);
            };
            let Some(size) = call_frame(cpu).read_typed::<usize, _>(*size) else {
                load_register_failed!(*size);
            };
            let Some(align) = call_frame(cpu).read_typed::<usize, _>(*align) else {
                load_register_failed!(*align);
            };
            (out, size, align, true)
        }
        Instruction_StackAllocate::Static { out, size, align } => {
            let Some(out) = call_frame(cpu).get_mut_typed::<NonNull<u8>, _>(*out) else {
                load_register_failed!(*out);
            };
            (out, (*size) as usize, (*align) as usize, false)
        }
        Instruction_StackAllocate::StaticZeroed { out, size, align } => {
            let Some(out) = call_frame(cpu).get_mut_typed::<NonNull<u8>, _>(*out) else {
                load_register_failed!(*out);
            };
            (out, (*size) as usize, (*align) as usize, true)
        }
    };

    let mut allocator = cpu.call_stack().current().unwrap().allocator().borrow_mut();
    let result = if should_zero {
        allocator.alloc_zeroed(size, align)
    } else {
        allocator.alloc_raw(size, align)
    };
    *out = result;

    Some(Ok(()))
}
