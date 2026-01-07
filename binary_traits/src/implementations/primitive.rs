use std::io::{Read, Write};

use types::{File, integer::CompressedU32};

use crate::{ReadFromFile, WriteToFile};

macro primitive_impl($($t:ty)+) {$(
    // impl super::ToBytes for $t {
    //     fn to_bytes(&self) -> [u8; <Self as super::ToBytes>::SIZE] {
    //         self.to_le_bytes()
    //     }
    // }
    // impl super::FromBytes for $t {
    //     fn from_bytes(bytes: [u8; <Self as super::FromBytes>::SIZE]) -> Self {
    //         Self::from_le_bytes(bytes)
    //     }
    // }
    impl super::super::ReadFromFile for $t {
        fn read_from_file(file: &mut ::types::File) -> global_errors::Result<Self> {
            let mut buf = [0u8; size_of::<$t>()];
            file.reader().read_exact(&mut buf)?;
            Ok(<$t>::from_le_bytes(buf))
        }
    }
    impl super::super::WriteToFile for $t {
        fn write_to_file(&self, file: &mut ::types::File) -> global_errors::Result<()> {
            file.writer().write_all(&self.to_le_bytes())?;
            Ok(())
        }
    }
)+}

primitive_impl! {
    u8
    u16
    u32
    u64
    u128
    i8
    i16
    i32
    i64
    i128
}

// impl super::ToBytes for bool {
//     fn to_bytes(&self) -> [u8; <Self as super::ToBytes>::SIZE] {
//         [(*self) as u8]
//     }
// }

impl WriteToFile for bool {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        file.writer().write_all(&[(*self) as u8])?;

        Ok(())
    }
}

// impl super::FromBytes for bool {
//     fn from_bytes(bytes: [u8; <Self as super::FromBytes>::SIZE]) -> Self {
//         debug_assert!(bytes[0] == 1 || bytes[0] == 0);
//         bytes[0] != 0
//     }
// }

impl ReadFromFile for bool {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let mut bytes = [0u8; 1];
        file.reader().read_exact(&mut bytes)?;
        debug_assert!(bytes[0] == 1 || bytes[0] == 0);
        Ok(bytes[0] != 0)
    }
}

// impl super::ToBytes for char {
//     const SIZE: usize = <u32 as super::ToBytes>::SIZE;
//     fn to_bytes(&self) -> [u8; <Self as super::ToBytes>::SIZE] {
//         ((*self) as u32).to_le_bytes()
//     }
// }

impl WriteToFile for char {
    /// Write [`char`] as [`u32`] to a file
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        (*self as u32).write_to_file(file)
    }
}

// impl super::FromBytes for char {
//     const SIZE: usize = <u32 as super::FromBytes>::SIZE;

//     fn from_bytes(bytes: [u8; <Self as super::FromBytes>::SIZE]) -> Self {
//         unsafe { char::from_u32_unchecked(u32::from_bytes(bytes)) }
//     }
// }

impl ReadFromFile for char {
    /// Read a '[Unicode scalar value]' and convert it to [`char`]
    /// using [`char::from_u32_unchecked`]
    ///
    /// [Unicode scalar value]: https://www.unicode.org/glossary/#unicode_scalar_value
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        u32::read_from_file(file).map(|i| unsafe { Self::from_u32_unchecked(i) })
    }
}

impl ReadFromFile for CompressedU32 {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let first = u8::read_from_file(file)?;

        // 1. 单字节格式
        if first <= 0x7F {
            return Ok(CompressedU32(first as u32));
        }

        // 2. 双字节格式
        if first <= 0xBF {
            let second = u8::read_from_file(file)? as u32;

            let value = ((first as u32 & 0x3F) << 8) | second;
            return Ok(CompressedU32(value));
        }

        // 3. 四字节格式
        if first <= 0xDF {
            let mut remaining_bytes = [0u8; 3];
            file.data.read_exact(&mut remaining_bytes)?;

            let byte2 = remaining_bytes[0] as u32;
            let byte3 = remaining_bytes[1] as u32;
            let byte4 = remaining_bytes[2] as u32;

            let value = ((first as u32 & 0x1F) << 24) | (byte2 << 16) | (byte3 << 8) | byte4;
            return Ok(CompressedU32(value));
        }

        Err(global_errors::BinaryError::IntOutOfRange.into())
    }
}

impl WriteToFile for CompressedU32 {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        match self.0 {
            x if x <= 0x7f => {
                file.data.write_all(&[self.0 as u8])?;
                Ok(())
            }
            x if x <= 0x3FFF => {
                let bytes = [0x80 | ((self.0 >> 8) as u8), (self.0 & 0xFF) as u8];
                file.data.write_all(&bytes)?;
                Ok(())
            }
            x if x <= 0x1FFFFFFF => {
                let bytes = [
                    0xC0 | ((self.0 >> 24) as u8),
                    ((self.0 >> 16) & 0xFF) as u8,
                    ((self.0 >> 8) & 0xFF) as u8,
                    (self.0 & 0xFF) as u8,
                ];
                file.data.write_all(&bytes)?;
                Ok(())
            }
            _ => Err(global_errors::BinaryError::IntOutOfRange.into()),
        }
    }
}
