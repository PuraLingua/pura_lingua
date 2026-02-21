use std::fmt::Display;

use binary_core::traits::{ReadFromSection, WriteToSection};
use bitfields::{FromBits, IntoBits};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JumpTargetType {
    Absolute = 0b00,
    Forward = 0b01,
    Backward = 0b10,
    Unknown,
}

impl std::fmt::Display for JumpTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl const FromBits for JumpTargetType {
    type Number = u8;
    fn from_bits(bits: Self::Number) -> Self {
        match bits {
            0b00 => Self::Absolute,
            0b01 => Self::Forward,
            0b10 => Self::Backward,
            _ => Self::Unknown,
        }
    }
}

impl const IntoBits for JumpTargetType {
    type Number = u8;

    fn into_bits(self) -> Self::Number {
        self as Self::Number
    }
}

#[bitfields::bitfield(u64, new = false, debug = true)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct JumpTarget {
    #[bits(2)]
    ty: JumpTargetType,
    #[bits(62)]
    val: u64,
}

impl Display for JumpTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}({:#x})", self.ty(), self.val()))
    }
}

impl ReadFromSection for JumpTarget {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&binary_core::section::Section>,
    ) -> binary_core::BinaryResult<Self> {
        u64::read_from_section(cursor).map(Self)
    }
}

impl WriteToSection for JumpTarget {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> binary_core::BinaryResult<()> {
        self.0.write_to_section(cursor)
    }
}
