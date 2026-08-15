use std::fmt::Display;

use binary_core::traits::{ReadFromSection, WriteToSection};

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

impl JumpTargetType {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => Self::Absolute,
            0b01 => Self::Forward,
            0b10 => Self::Backward,
            _ => Self::Unknown,
        }
    }
}

impl JumpTargetType {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }
}

#[bitfields::bitfield(u64, new = false, debug = true)]
#[derive(PartialEq, Eq)]
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
