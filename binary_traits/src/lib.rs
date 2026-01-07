#![feature(decl_macro)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(macro_metavar_expr)]

use types::File;
// use std::io::{Read, Write};

mod implementations;

pub trait WriteToFile {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()>;
}

pub trait ReadFromFile: Sized {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self>;
}

// pub(crate) trait ToBytes: Copy {
//     const SIZE: usize = size_of::<Self>();
//     fn to_bytes(&self) -> [u8; <Self as ToBytes>::SIZE];
// }

// pub(crate) trait FromBytes: Copy {
//     const SIZE: usize = size_of::<Self>();
//     fn from_bytes(bytes: [u8; <Self as FromBytes>::SIZE]) -> Self;
// }

// impl<T: ToBytes> WriteToFile for T
// where
//     [(); <Self as ToBytes>::SIZE]:,
// {
//     fn write_to_file(&self, file: &mut File) -> global::Result<()> {
//         file.writer().write_all(&self.to_bytes())?;
//         Ok(())
//     }
// }

// impl<T: FromBytes> ReadFromFile for T
// where
//     [(); <Self as FromBytes>::SIZE]:,
// {
//     fn read_from_file(file: &mut File) -> global::Result<Self> {
//         let mut buf = [0u8; <Self as FromBytes>::SIZE];
//         file.reader().read_exact(&mut buf)?;
//         Ok(Self::from_bytes(buf))
//     }
// }
