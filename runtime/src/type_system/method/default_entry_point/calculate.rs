use std::{
    ffi::c_void,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
    ptr::NonNull,
};

use global::instruction::{IRegisterAddr, Instruction_Calculate, Instruction_UntypedCalculate};

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
    ins: &Instruction_Calculate<TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    match ins {
        Instruction_Calculate::U8(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }
        Instruction_Calculate::U16(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }
        Instruction_Calculate::U32(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }
        Instruction_Calculate::U64(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }

        Instruction_Calculate::I8(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }
        Instruction_Calculate::I16(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }
        Instruction_Calculate::I32(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }
        Instruction_Calculate::I64(ins) => {
            eval_untyped(method, cpu, this, args, result_ptr, pc, ins)
        }
    }
}

trait ConstsForEval:
    Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + RemAssign
{
    const ONE: Self;
}

macro impl_consts_for_eval($($i:ty),* $(,)?) {$(
	impl ConstsForEval for $i {
		const ONE: Self = 1;
	}
)*}

impl_consts_for_eval!(u8, u16, u32, u64, i8, i16, i32, i64);

#[allow(private_bounds)]
pub(super) fn eval_untyped<
    T: Sized + GetAssemblyRef + GetTypeVars,
    TRegisterAddr: IRegisterAddr,
    TRust: ConstsForEval,
>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    ins: &Instruction_UntypedCalculate<TRegisterAddr, TRust>,
) -> Option<Result<(), Termination>> {
    macro read($target:ident = ($TRust:ty)$i:ident) {
        let Some($target) = call_frame(cpu).read_typed::<$TRust, _>(*$i) else {
            load_register_failed!(*$i);
        };
    }
    macro write_reg($target:ident = $i:expr) {
        if !call_frame(cpu).write_typed(*$target, $i) {
            load_register_failed!(*$target);
        }
    }
    macro get($target:ident = ($TRust:ty)$i:ident) {
        let Some($target) = call_frame(cpu).get_mut_typed::<$TRust, _>(*$i) else {
            load_register_failed!(*$i);
        };
    }
    match ins {
        Instruction_UntypedCalculate::Add { lhs, rhs, target } => {
            read!(lhs = (TRust)lhs);
            read!(rhs = (TRust)rhs);
            let res = lhs + rhs;
            write_reg!(target = res);
        }
        Instruction_UntypedCalculate::Sub { lhs, rhs, target } => {
            read!(lhs = (TRust)lhs);
            read!(rhs = (TRust)rhs);
            let res = lhs - rhs;
            write_reg!(target = res);
        }
        Instruction_UntypedCalculate::Mul { lhs, rhs, target } => {
            read!(lhs = (TRust)lhs);
            read!(rhs = (TRust)rhs);
            let res = lhs * rhs;
            write_reg!(target = res);
        }
        Instruction_UntypedCalculate::Div { lhs, rhs, target } => {
            read!(lhs = (TRust)lhs);
            read!(rhs = (TRust)rhs);
            let res = lhs / rhs;
            write_reg!(target = res);
        }
        Instruction_UntypedCalculate::Rem { lhs, rhs, target } => {
            read!(lhs = (TRust)lhs);
            read!(rhs = (TRust)rhs);
            let res = lhs % rhs;
            write_reg!(target = res);
        }

        Instruction_UntypedCalculate::ConstAddTo { target, data } => {
            get!(target = (TRust)target);
            target.add_assign(*data);
        }
        Instruction_UntypedCalculate::ConstSubTo { target, data } => {
            get!(target = (TRust)target);
            target.sub_assign(*data);
        }
        Instruction_UntypedCalculate::ConstMulTo { target, data } => {
            get!(target = (TRust)target);
            target.mul_assign(*data);
        }
        Instruction_UntypedCalculate::ConstDivTo { target, data } => {
            get!(target = (TRust)target);
            target.div_assign(*data);
        }
        Instruction_UntypedCalculate::ConstRemTo { target, data } => {
            get!(target = (TRust)target);
            target.rem_assign(*data);
        }

        Instruction_UntypedCalculate::SubByConst { target, data } => {
            get!(target = (TRust)target);
            *target = (*data).sub(*target);
        }
        Instruction_UntypedCalculate::DivByConst { target, data } => {
            get!(target = (TRust)target);
            *target = (*data).div(*target);
        }
        Instruction_UntypedCalculate::RemByConst { target, data } => {
            get!(target = (TRust)target);
            *target = (*data).rem(*target);
        }

        Instruction_UntypedCalculate::AddOne { target } => {
            get!(target = (TRust)target);
            target.add_assign(TRust::ONE);
        }
        Instruction_UntypedCalculate::SubOne { target } => {
            get!(target = (TRust)target);
            target.sub_assign(TRust::ONE);
        }
    }

    Some(Ok(()))
}
