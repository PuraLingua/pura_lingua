use std::{ffi::c_void, ptr::NonNull};

use global::instruction::{IRegisterAddr, Instruction_CommonCheck, ToCheckContent};

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

pub(super) fn eval<T: Sized + GetAssemblyRef + GetTypeVars, TRegisterAddr: IRegisterAddr>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    #[allow(unused)] caught_exception: Option<ManagedReference<Class>>,
    ins: &Instruction_CommonCheck<TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    let register_addr = &ins.output;

    let res = match eval_to_check(method, cpu, this, args, result_ptr, pc, &ins.content)? {
        Ok(x) => x,
        Err(ter) => return Some(Err(ter)),
    };

    if !call_frame(cpu).write_typed(*register_addr, res) {
        load_register_failed!(*register_addr);
    }

    Some(Ok(()))
}

pub(super) fn eval_to_check<
    T: Sized + GetAssemblyRef + GetTypeVars,
    TRegisterAddr: IRegisterAddr,
>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    to_check: &ToCheckContent<TRegisterAddr>,
) -> Option<Result<bool, Termination>> {
    match &to_check {
        ToCheckContent::IsAllZero(to_check) => {
            let Some(to_check_var) = call_frame(cpu).get(*to_check) else {
                load_register_failed!(*to_check);
            };
            Some(Ok(to_check_var.is_all_zero()))
        }
    }
}
