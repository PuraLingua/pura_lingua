use std::io::Cursor;

use crate::{BinaryResult, section::Section};

#[cfg(test)]
mod derive_test;
mod implementations;

pub use implementations::StringRef;

pub trait ReadFromSection: Sized {
    fn read_from_section(cursor: &mut Cursor<&Section>) -> BinaryResult<Self>;
}

pub trait WriteToSection {
    fn write_to_section(&self, cursor: &mut Cursor<&mut Vec<u8>>) -> BinaryResult<()>;
}

macro_rules! tuple_read_impl {
    () => {
        impl ReadFromSection for () {
            #[optimize(size)]
            #[inline(always)]
            fn read_from_section(_: &mut Cursor<&Section>) -> BinaryResult<Self> {
                Ok(())
            }
        }
    };
    ($i:ident) => {
        impl<$i> ReadFromSection for ($i,)
        where
            $i: ReadFromSection,
        {
            #[optimize(size)]
            fn read_from_section(cursor: &mut Cursor<&Section>) -> BinaryResult<Self> {
                $i::read_from_section(cursor).map(|x| (x,))
            }
        }
        tuple_read_impl!();
    };
    ($i:ident $($rest:ident)+) => {
        #[allow(nonstandard_style)]
        impl<$i $(,$rest)+> ReadFromSection for ($i $(,$rest)+)
        where
            $i: ReadFromSection,
            $(
                $rest: ReadFromSection,
            )+
        {
            #[optimize(size)]
            fn read_from_section(cursor: &mut Cursor<&Section>) -> BinaryResult<Self> {
                let first = $i::read_from_section(cursor)?;
                $(
                    let $rest = $rest::read_from_section(cursor)?;
                )+
                Ok((first $(,$rest)+))
            }
        }
        tuple_read_impl!($($rest)+);
    };
}

macro_rules! tuple_write_impl {
    () => {
        impl WriteToSection for () {
            #[optimize(size)]
            fn write_to_section(&self, _: &mut Cursor<&mut Vec<u8>>) -> BinaryResult<()> {
                Ok(())
            }
        }
    };
    ($i:ident) => {
        impl<$i> WriteToSection for ($i,)
        where
            $i: WriteToSection,
        {
            #[optimize(size)]
            fn write_to_section(&self, cursor: &mut Cursor<&mut Vec<u8>>) -> BinaryResult<()> {
                self.0.write_to_section(cursor)
            }
        }
        tuple_write_impl!();
    };
    ($i:ident $($rest:ident)+) => {
        #[allow(nonstandard_style)]
        impl<$i $(,$rest)+> WriteToSection for ($i $(,$rest)+)
        where
            $i: WriteToSection,
            $(
                $rest: WriteToSection,
            )+
        {
            #[optimize(size)]
            fn write_to_section(&self, cursor: &mut Cursor<&mut Vec<u8>>) -> BinaryResult<()> {
                let ($i $(,$rest)+) = self;
                $i.write_to_section(cursor)?;
                $($rest.write_to_section(cursor)?;)+
                Ok(())
            }
        }
        tuple_write_impl!($($rest)+);
    };
}

tuple_read_impl! (
    A B
    // C D E F G H I J K L M N O P Q R S T U V W X Y Z
);
tuple_write_impl!(
    A B
    // C D E F G H I J K L M N O P Q R S T U V W X Y Z
);
