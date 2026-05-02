use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};
use global_proc_macros::{DeriveMap, Transpose, WithType};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::IRegisterAddr;

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection, Transpose, DeriveMap)]
#[transpose(TTypeRef, TFieldRef)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_Set<TTypeRef, TFieldRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
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

impl<TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr> Display
    for Instruction_Set<TTypeRef, TFieldRef, TRegisterAddr>
where
    TTypeRef: Display,
    TFieldRef: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction_Set::Common {
                val,
                container,
                field,
            } => f.write_fmt(format_args!(" {val:#x} -> {container}.{field}")),
            Instruction_Set::This { val, field } => {
                f.write_fmt(format_args!(" {val:#x} -> this.{field}"))
            }
            Instruction_Set::Static { val, ty, field } => {
                f.write_fmt(format_args!("Static {val:#x} -> {ty}.{field}"))
            }
        }
    }
}
