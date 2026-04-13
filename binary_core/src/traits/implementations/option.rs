use std::{
    io::Write,
    num::{NonZero, ZeroablePrimitive},
};

use crate::traits::{ReadFromSection, WriteToSection};

impl<T: ReadFromSection> ReadFromSection for Option<T> {
    default fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        match bool::read_from_section(cursor)? {
            true => T::read_from_section(cursor).map(Some),
            false => Ok(None),
        }
    }
}

impl<T: WriteToSection> WriteToSection for Option<T> {
    default fn write_to_section(
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

impl<T: ReadFromSection> ReadFromSection for Option<NonZero<T>>
where
    T: ZeroablePrimitive,
{
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> crate::BinaryResult<Self> {
        T::read_from_section(cursor).map(NonZero::new)
    }
}

impl<T: WriteToSection> WriteToSection for Option<NonZero<T>>
where
    T: ZeroablePrimitive,
{
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> crate::BinaryResult<()> {
        match self {
            Some(x) => x.get().write_to_section(cursor),
            None => cursor
                .write_all(&vec![0; size_of::<NonZero<T>>()])
                .map_err(From::from),
        }
    }
}
