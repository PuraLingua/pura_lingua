use std::{ffi::c_void, ptr::NonNull};

use global::instruction::{
    IRegisterAddr, Instruction_Jump, JumpCondition, JumpTarget, JumpTargetType,
};

use crate::{
    type_system::{
        get_traits::{GetAssemblyRef, GetTypeVars},
        method::{
            Method,
            default_entry_point::{Termination, call_frame, load_register_failed},
        },
    },
    virtual_machine::cpu::CPU,
};

fn do_jump(pc: &mut usize, target: JumpTarget) {
    match target.ty() {
        JumpTargetType::Absolute => {
            *pc = (target.val() as usize) - 1;
        }
        JumpTargetType::Forward => {
            <_ as std::ops::AddAssign>::add_assign(pc, (target.val() as usize) - 1);
        }
        JumpTargetType::Backward => {
            <_ as std::ops::SubAssign>::sub_assign(pc, (target.val() as usize) - 1);
        }
        JumpTargetType::Unknown => unreachable!(),
    }
}

pub(super) fn eval<T: Sized + GetAssemblyRef + GetTypeVars, TRegisterAddr: IRegisterAddr>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    ins: &Instruction_Jump<TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    let cond = match &ins.condition {
        JumpCondition::Unconditional => true,
        JumpCondition::If(cond) => {
            let Some(cond) = call_frame(cpu).get_typed::<bool, _>(*cond) else {
                load_register_failed!(*cond);
            };
            *cond
        }
        JumpCondition::IfCheckSucceeds(to_check) => {
            match super::check::eval_to_check(method, cpu, this, args, result_ptr, pc, to_check)? {
                Ok(x) => x,
                Err(ter) => return Some(Err(ter)),
            }
        }
        JumpCondition::IfCheckFails(to_check) => {
            match super::check::eval_to_check(method, cpu, this, args, result_ptr, pc, to_check)? {
                Ok(x) => !x,
                Err(ter) => return Some(Err(ter)),
            }
        }
    };
    if cond {
        do_jump(pc, ins.target);
    }

    Some(Ok(()))
}
