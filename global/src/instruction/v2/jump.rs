use binary_proc_macros::{ReadFromSection, WriteToSection};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::WithType;

use crate::instruction::v2::{IRegisterAddr, JumpTarget, ToCheckContent};

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct Instruction_Jump<TRegisterAddr: IRegisterAddr> {
    pub target: JumpTarget,
    pub condition: JumpCondition<TRegisterAddr>,
}

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum JumpCondition<TRegisterAddr: IRegisterAddr> {
    Unconditional,
    If(TRegisterAddr),
    IfCheckSucceeds(ToCheckContent<TRegisterAddr>),
    IfCheckFails(ToCheckContent<TRegisterAddr>),
}
