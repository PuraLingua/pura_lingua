use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};
use global_proc_macros::WithType;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{IRegisterAddr, JumpTarget, RegisterAddr, ShortRegisterAddr, ToCheckContent};

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct Instruction_Jump<TRegisterAddr: IRegisterAddr> {
    pub target: JumpTarget,
    pub condition: JumpCondition<TRegisterAddr>,
}

impl Instruction_Jump<RegisterAddr> {
    pub fn try_into_short(self) -> Result<Instruction_Jump<ShortRegisterAddr>, Self> {
        let Instruction_Jump { target, condition } = self;
        match condition.try_into_short() {
            Ok(condition) => Ok(Instruction_Jump { target, condition }),
            Err(condition) => Err(Self { target, condition }),
        }
    }
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

impl JumpCondition<RegisterAddr> {
    pub fn try_into_short(self) -> Result<JumpCondition<ShortRegisterAddr>, Self> {
        match self {
            Self::Unconditional => Ok(JumpCondition::Unconditional),
            Self::If(cond) => {
                if let Some(cond) = cond.try_into_short() {
                    Ok(JumpCondition::If(cond))
                } else {
                    Err(Self::If(cond))
                }
            }
            Self::IfCheckSucceeds(to_check) => match to_check.try_to_short() {
                Ok(to_check) => Ok(JumpCondition::IfCheckSucceeds(to_check)),
                Err(to_check) => Err(Self::IfCheckSucceeds(to_check)),
            },
            Self::IfCheckFails(to_check) => match to_check.try_to_short() {
                Ok(to_check) => Ok(JumpCondition::IfCheckFails(to_check)),
                Err(to_check) => Err(Self::IfCheckFails(to_check)),
            },
        }
    }
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
