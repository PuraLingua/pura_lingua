use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};

use global_proc_macros::WithType;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct Instruction_CommonCheck<TRegisterAddr: IRegisterAddr> {
    pub output: TRegisterAddr,
    pub content: ToCheckContent<TRegisterAddr>,
}

impl Instruction_Check {
    pub const fn try_into_short(self) -> Result<Instruction_SCheck, Self> {
        let Instruction_CommonCheck { output, content } = self;
        let Some(output) = output.try_into_short() else {
            return Err(Instruction_CommonCheck { output, content });
        };
        match content.try_to_short() {
            Ok(content) => Ok(Instruction_CommonCheck { output, content }),
            Err(content) => Err(Instruction_CommonCheck {
                output: output.into_generic(),
                content,
            }),
        }
    }
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
    pub const fn try_to_short(self) -> Result<ToCheckContent<ShortRegisterAddr>, Self> {
        match self {
            ToCheckContent::IsAllZero(to_check) => match to_check.try_into_short() {
                Some(to_check) => Ok(ToCheckContent::IsAllZero(to_check)),
                None => Err(self),
            },
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
