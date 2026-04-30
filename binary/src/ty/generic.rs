use std::{
    io::Read,
    range::{RangeFrom, RangeToInclusive},
};

use binary_core::{
    Error,
    traits::{ReadFromSection, WriteToSection},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenericCountRequirement {
    AtLeast(RangeFrom<u32>),
    NoMoreThan(RangeToInclusive<u32>),
    Exact(u32),
}

impl ReadFromSection for GenericCountRequirement {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&binary_core::section::Section>,
    ) -> binary_core::BinaryResult<Self> {
        let mut kind = 0u8;
        cursor.read_exact(std::slice::from_mut(&mut kind))?;

        match kind {
            0x00 => {
                let start = u32::read_from_section(cursor)?;
                Ok(Self::AtLeast(RangeFrom { start }))
            }
            0x01 => {
                let last = u32::read_from_section(cursor)?;
                Ok(Self::NoMoreThan(RangeToInclusive { last }))
            }
            0x02 => {
                let val = u32::read_from_section(cursor)?;
                Ok(Self::Exact(val))
            }
            _ => Err(Error::IndexOutOfRange),
        }
    }
}

impl WriteToSection for GenericCountRequirement {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> binary_core::BinaryResult<()> {
        match self {
            GenericCountRequirement::AtLeast(range_from) => {
                0x00u8.write_to_section(cursor)?;
                range_from.start.write_to_section(cursor)?;
                Ok(())
            }
            GenericCountRequirement::NoMoreThan(range_to_inclusive) => {
                0x01u8.write_to_section(cursor)?;
                range_to_inclusive.last.write_to_section(cursor)?;
                Ok(())
            }
            GenericCountRequirement::Exact(val) => {
                0x02u8.write_to_section(cursor)?;
                val.write_to_section(cursor)?;
                Ok(())
            }
        }
    }
}
