use std::io::{Read, Write};

use enumflags2::{BitFlag, BitFlags};

use crate::{
    CompressedU32,
    traits::{ReadFromSection, WriteToSection},
};

impl super::super::ReadFromSection for u8 {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        let mut this = 0u8;
        cursor.read_exact(std::slice::from_mut(&mut this))?;

        Ok(this)
    }
}

impl super::super::WriteToSection for u8 {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        cursor.write_all(std::slice::from_ref(self))?;

        Ok(())
    }
}

impl super::super::ReadFromSection for i8 {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        let mut this = 0i8;
        unsafe {
            cursor.read_exact(&mut *(&raw mut this as *mut [i8; 1] as *mut [u8; 1]))?;
        }

        Ok(this)
    }
}

impl super::super::WriteToSection for i8 {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        unsafe {
            cursor.write_all(&*(&raw const self as *const [i8; 1] as *const [u8; 1]))?;
        }

        Ok(())
    }
}

macro _impl($($T:ty)*) {$(
	impl $crate::traits::ReadFromSection for $T {
		fn read_from_section(
			cursor: &mut std::io::Cursor<&crate::section::Section>,
		) -> Result<Self, crate::error::Error> {
			let this = cursor.read_array::<{ size_of::<Self>() }>()?;
			Ok(Self::from_le_bytes(this))
		}
	}
	impl $crate::traits::WriteToSection for $T {
		fn write_to_section(&self, cursor: &mut std::io::Cursor<&mut Vec<u8>>) -> Result<(), crate::error::Error> {
			cursor.write_all(&self.to_le_bytes())?;

			Ok(())
		}
	}
)*}

_impl! {
    u16
    u32
    u64
    i16
    i32
    i64
}

impl ReadFromSection for bool {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        u8::read_from_section(cursor).map(|x| x != 0)
    }
}

impl WriteToSection for bool {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        (*self as u8).write_to_section(cursor)
    }
}

impl ReadFromSection for char {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        u32::read_from_section(cursor)
            .map(Self::from_u32)
            .and_then(|x| x.ok_or(crate::error::Error::InvalidChar))
    }
}

impl WriteToSection for char {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        (*self as u32).write_to_section(cursor)
    }
}

impl<T: BitFlag> ReadFromSection for BitFlags<T>
where
    <T as enumflags2::_internal::RawBitFlags>::Numeric: ReadFromSection,
{
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        let raw = <T as enumflags2::_internal::RawBitFlags>::Numeric::read_from_section(cursor)?;
        Ok(BitFlags::from_bits_truncate(raw))
    }
}

impl<T: BitFlag> WriteToSection for BitFlags<T>
where
    <T as enumflags2::_internal::RawBitFlags>::Numeric: WriteToSection,
{
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        self.bits().write_to_section(cursor)?;

        Ok(())
    }
}

impl ReadFromSection for CompressedU32 {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        let first = u8::read_from_section(cursor)?;

        // 1 byte
        if first <= 0x7F {
            return Ok(CompressedU32(first as u32));
        }

        // 2 bytes
        if first <= 0xBF {
            let second = u8::read_from_section(cursor)? as u32;

            let value = ((first as u32 & 0x3F) << 8) | second;
            return Ok(CompressedU32(value));
        }

        // 4 bytes
        if first <= 0xDF {
            let mut remaining_bytes = [0u8; 3];
            cursor.read_exact(&mut remaining_bytes)?;

            let byte2 = remaining_bytes[0] as u32;
            let byte3 = remaining_bytes[1] as u32;
            let byte4 = remaining_bytes[2] as u32;

            let value = ((first as u32 & 0x1F) << 24) | (byte2 << 16) | (byte3 << 8) | byte4;
            return Ok(CompressedU32(value));
        }

        Err(crate::error::Error::IntegerOutOfRange)
    }
}

impl WriteToSection for CompressedU32 {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        match self.0 {
            x if x <= 0x7f => {
                cursor.write_all(&[self.0 as u8])?;
                Ok(())
            }
            x if x <= 0x3FFF => {
                let bytes = [0x80 | ((self.0 >> 8) as u8), (self.0 & 0xFF) as u8];
                cursor.write_all(&bytes)?;
                Ok(())
            }
            x if x <= 0x1FFFFFFF => {
                let bytes = [
                    0xC0 | ((self.0 >> 24) as u8),
                    ((self.0 >> 16) & 0xFF) as u8,
                    ((self.0 >> 8) & 0xFF) as u8,
                    (self.0 & 0xFF) as u8,
                ];
                cursor.write_all(&bytes)?;
                Ok(())
            }
            _ => Err(crate::error::Error::IntegerOutOfRange),
        }
    }
}
