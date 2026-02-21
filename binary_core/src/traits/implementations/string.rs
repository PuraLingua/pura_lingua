use crate::{
    BinaryResult,
    traits::{ReadFromSection, WriteToSection},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringRef(pub(crate) u64);

impl ReadFromSection for StringRef {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> BinaryResult<Self> {
        u64::read_from_section(cursor).map(Self)
    }
}

impl WriteToSection for StringRef {
    fn write_to_section(&self, cursor: &mut std::io::Cursor<&mut Vec<u8>>) -> BinaryResult<()> {
        self.0.write_to_section(cursor)
    }
}
