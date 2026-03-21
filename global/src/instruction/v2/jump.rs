use std::fmt::Display;

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

impl<TRegisterAddr: IRegisterAddr> Display for Instruction_Jump<TRegisterAddr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(" {}", self.target))?;

        match &self.condition {
            JumpCondition::Unconditional => Ok(()),
            JumpCondition::If(cond) => f.write_fmt(format_args!(" if {cond:#x}")),
            JumpCondition::IfCheckSucceeds(to_check) => {
                f.write_fmt(format_args!(" if {to_check} succeeds"))
            }
            JumpCondition::IfCheckFails(to_check) => {
                f.write_fmt(format_args!(" if {to_check} fails"))
            }
        }
    }
}
