use binary_proc_macros::{ReadFromSection, WriteToSection};
use global_proc_macros::WithType;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[derive(Clone, Copy, Debug, ReadFromSection, WriteToSection, WithType)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(repr = u8)]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_StackAllocate<TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    Dynamic {
        out: TRegisterAddr,
        size: TRegisterAddr,
        align: TRegisterAddr,
    },
    DynamicZeroed {
        out: TRegisterAddr,
        size: TRegisterAddr,
        align: TRegisterAddr,
    },
    Static {
        out: TRegisterAddr,
        size: u64,
        align: u64,
    },
    StaticZeroed {
        out: TRegisterAddr,
        size: u64,
        align: u64,
    },
}

impl Instruction_StackAllocate<RegisterAddr> {
    pub fn try_into_short(self) -> Result<Instruction_StackAllocate<ShortRegisterAddr>, Self> {
        match self {
            Instruction_StackAllocate::Dynamic { out, size, align } => {
                if let Some(out) = out.try_into_short()
                    && let Some(size) = size.try_into_short()
                    && let Some(align) = align.try_into_short()
                {
                    Ok(Instruction_StackAllocate::Dynamic { out, size, align })
                } else {
                    Err(self)
                }
            }
            Instruction_StackAllocate::DynamicZeroed { out, size, align } => {
                if let Some(out) = out.try_into_short()
                    && let Some(size) = size.try_into_short()
                    && let Some(align) = align.try_into_short()
                {
                    Ok(Instruction_StackAllocate::DynamicZeroed { out, size, align })
                } else {
                    Err(self)
                }
            }
            Instruction_StackAllocate::Static { out, size, align } => {
                if let Some(out) = out.try_into_short() {
                    Ok(Instruction_StackAllocate::Static { out, size, align })
                } else {
                    Err(self)
                }
            }
            Instruction_StackAllocate::StaticZeroed { out, size, align } => {
                if let Some(out) = out.try_into_short() {
                    Ok(Instruction_StackAllocate::StaticZeroed { out, size, align })
                } else {
                    Err(self)
                }
            }
        }
    }
}

impl<TRegisterAddr: IRegisterAddr> std::fmt::Display for Instruction_StackAllocate<TRegisterAddr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction_StackAllocate::Dynamic { out, size, align } => {
                write!(f, " size at {size:#x}, align at {align:#x} -> {out:#x}")
            }
            Instruction_StackAllocate::DynamicZeroed { out, size, align } => {
                write!(
                    f,
                    "Zeroed size at {size:#x}, align at {align:#x} -> {out:#x}"
                )
            }
            Instruction_StackAllocate::Static { out, size, align } => {
                write!(
                    f,
                    " size: {size}({size:#x}), align: {align}({align:#x}) -> {out:#x}"
                )
            }
            Instruction_StackAllocate::StaticZeroed { out, size, align } => {
                write!(
                    f,
                    "Zeroed size: {size}({size:#x}), align: {align}({align:#x}) -> {out:#x}"
                )
            }
        }
    }
}
