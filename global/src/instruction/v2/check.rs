use binary_proc_macros::{ReadFromSection, WriteToSection};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::WithType;

use crate::instruction::v2::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct Instruction_Check {
    pub output: RegisterAddr,
    pub content: ToCheckContent<RegisterAddr>,
}

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct Instruction_SCheck {
    pub output: ShortRegisterAddr,
    pub content: ToCheckContent<ShortRegisterAddr>,
}

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum ToCheckContent<TRegisterAddr: IRegisterAddr> {
    IsAllZero(TRegisterAddr),
}
