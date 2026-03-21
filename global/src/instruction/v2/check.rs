use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::WithType;

use crate::instruction::v2::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct Instruction_CommonCheck<TRegisterAddr: IRegisterAddr> {
    pub output: TRegisterAddr,
    pub content: ToCheckContent<TRegisterAddr>,
}

pub type Instruction_Check = Instruction_CommonCheck<RegisterAddr>;
pub type Instruction_SCheck = Instruction_CommonCheck<ShortRegisterAddr>;

impl<TRegisterAddr: IRegisterAddr> Display for Instruction_CommonCheck<TRegisterAddr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(" {} -> {}", self.content, self.output))
    }
}

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum ToCheckContent<TRegisterAddr: IRegisterAddr> {
    IsAllZero(TRegisterAddr),
}

impl ToCheckContent<RegisterAddr> {
    pub fn try_to_short(&self) -> Option<ToCheckContent<ShortRegisterAddr>> {
        match self {
            ToCheckContent::IsAllZero(to_check) => {
                to_check.try_into_short().map(ToCheckContent::IsAllZero)
            }
        }
    }
}

impl<TRegisterAddr: IRegisterAddr> Display for ToCheckContent<TRegisterAddr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToCheckContent::IsAllZero(to_check) => {
                f.write_fmt(format_args!("is_all_zero({to_check:#x})"))
            }
        }
    }
}
