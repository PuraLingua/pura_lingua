#![allow(non_camel_case_types)]

use std::ptr::NonNull;

use binary_proc_macros::{ReadFromSection, WriteToSection};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use proc_macros::WithType;

mod jumping;
pub use jumping::*;

mod register_addr;
pub use register_addr::*;

mod call;
mod check;
mod jump;
mod load;
mod new;
mod read_write_pointer;
mod set;

pub use call::*;
pub use check::*;
pub use jump::*;
pub use load::*;
pub use new::*;
pub use read_write_pointer::*;
pub use set::*;

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction<TString, TTypeRef, TMethodRef, TFieldRef> {
    Load(Instruction_Load<TString, TTypeRef, TFieldRef>),
    SLoad(Instruction_SLoad<TString, TTypeRef, TFieldRef>),

    ReadPointerTo(ReadPointerTo),
    SReadPointerTo(SReadPointerTo),

    WritePointer(WritePointer),
    SWritePointer(SWritePointer),

    Check(Instruction_Check),
    SCheck(Instruction_SCheck),

    New(Instruction_New<TTypeRef, TMethodRef, RegisterAddr>),
    SNew(Instruction_New<TTypeRef, TMethodRef, ShortRegisterAddr>),

    Call(Instruction_Call<TTypeRef, TMethodRef, RegisterAddr>),
    SCall(Instruction_Call<TTypeRef, TMethodRef, ShortRegisterAddr>),

    Set(Instruction_Set<TTypeRef, TFieldRef, RegisterAddr>),
    SSet(Instruction_Set<TTypeRef, TFieldRef, ShortRegisterAddr>),

    Throw { exception_addr: RegisterAddr },
    SThrow { exception_addr: ShortRegisterAddr },

    SReturnVal { register_addr: ShortRegisterAddr },

    Jump(Instruction_Jump<RegisterAddr>),
    SJump(Instruction_Jump<ShortRegisterAddr>),
}

impl<TString, TTypeRef, TMethodRef, TFieldRef>
    Instruction<TString, TTypeRef, TMethodRef, TFieldRef>
{
    pub fn type_ptr(&self) -> NonNull<u8> {
        NonNull::from_ref(self).cast()
    }
    pub fn data_ptr(&self) -> NonNull<()> {
        unsafe {
            NonNull::from_ref(self)
                .cast::<()>()
                .byte_add(size_of::<u8>())
        }
    }
}
