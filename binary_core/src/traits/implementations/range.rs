use std::range::Range;

use crate::traits::{ReadFromSection, WriteToSection};

impl<T: ReadFromSection> ReadFromSection for Range<T> {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> crate::BinaryResult<Self> {
        let start = T::read_from_section(cursor)?;
        let end = T::read_from_section(cursor)?;
        Ok(Self { start, end })
    }
}

impl<T: WriteToSection> WriteToSection for Range<T> {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> crate::BinaryResult<()> {
        let Self { start, end } = self;
        start.write_to_section(cursor)?;
        end.write_to_section(cursor)?;
        Ok(())
    }
}
