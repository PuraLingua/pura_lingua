use crate::traits::{ReadFromSection, WriteToSection};

impl<T: ReadFromSection> ReadFromSection for Option<T> {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        match bool::read_from_section(cursor)? {
            true => T::read_from_section(cursor).map(Some),
            false => Ok(None),
        }
    }
}

impl<T: WriteToSection> WriteToSection for Option<T> {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        match self {
            Self::Some(s) => true
                .write_to_section(cursor)
                .and_then(|_| s.write_to_section(cursor)),
            Self::None => false.write_to_section(cursor),
        }
    }
}
