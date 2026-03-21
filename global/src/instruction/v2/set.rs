use binary_proc_macros::{ReadFromSection, WriteToSection};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::WithType;

use crate::instruction::v2::IRegisterAddr;

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_Set<TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr> {
    Common {
        val: TRegisterAddr,
        container: TRegisterAddr,
        field: TFieldRef,
    },
    This {
        val: TRegisterAddr,
        field: TFieldRef,
    },
    Static {
        val: TRegisterAddr,
        ty: TTypeRef,
        field: TFieldRef,
    },
}
