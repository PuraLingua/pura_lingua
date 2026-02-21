use std::{
    alloc::Layout,
    ffi::{c_char, c_schar, c_uchar},
    ptr::NonNull,
};

use binary_core::traits::{ReadFromSection, WriteToSection};
use binary_proc_macros::{ReadFromSection, WriteToSection};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::attrs::CallConvention;

#[repr(C)]
#[derive(ReadFromSection, WriteToSection, Debug, Clone, PartialEq, Eq)]
pub struct NonPurusCallConfiguration {
    pub call_convention: CallConvention,
    pub return_type: NonPurusCallType,
    pub encoding: StringEncoding,
    pub object_strategy: ObjectStrategy,
    pub arguments: Vec<(/* is by ref */ bool, NonPurusCallType)>,
}

#[repr(u8)]
#[derive(
    ReadFromSection,
    WriteToSection,
    Clone,
    Copy,
    Default,
    Debug,
    TryFromPrimitive,
    IntoPrimitive,
    PartialEq,
    Eq,
)]
#[allow(nonstandard_style)]
pub enum StringEncoding {
    Utf16,
    Utf8,
    C_Utf16,
    C_Utf8,

    #[default]
    /// Performs the object_strategy
    Remain,
}

#[repr(u8)]
#[derive(
    ReadFromSection,
    WriteToSection,
    Clone,
    Copy,
    Default,
    Debug,
    TryFromPrimitive,
    IntoPrimitive,
    PartialEq,
    Eq,
)]
#[allow(nonstandard_style)]
pub enum ObjectStrategy {
    #[default]
    Remain,
    PointToData,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NonPurusCallType {
    /// For return type only
    Void,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    String,
    Object,
    Structure(Vec<NonPurusCallType>) = Self::STRUCTURE_DISCRIMINANT,
}

impl NonPurusCallType {
    pub const STRUCTURE_DISCRIMINANT: u8 = 0xff;
    pub const fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
    pub fn from_u8<F: FnOnce() -> Option<Vec<NonPurusCallType>>>(
        discriminant: u8,
        field_producer: F,
    ) -> Option<Self> {
        match discriminant {
            x if x == Self::Void.discriminant() => Some(Self::Void),
            x if x == Self::U8.discriminant() => Some(Self::U8),
            x if x == Self::I8.discriminant() => Some(Self::I8),
            x if x == Self::U16.discriminant() => Some(Self::U16),
            x if x == Self::I16.discriminant() => Some(Self::I16),
            x if x == Self::U32.discriminant() => Some(Self::U32),
            x if x == Self::I32.discriminant() => Some(Self::I32),
            x if x == Self::U64.discriminant() => Some(Self::U64),
            x if x == Self::I64.discriminant() => Some(Self::I64),
            x if x == Self::String.discriminant() => Some(Self::String),
            x if x == Self::Object.discriminant() => Some(Self::Object),

            Self::STRUCTURE_DISCRIMINANT => field_producer().map(Self::Structure),

            _ => None,
        }
    }
    pub fn as_parts(&self) -> (u8, Option<&Vec<NonPurusCallType>>) {
        match self {
            NonPurusCallType::Structure(fields) => (self.discriminant(), Some(fields)),

            _ => (self.discriminant(), None),
        }
    }
}

impl ReadFromSection for NonPurusCallType {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&binary_core::section::Section>,
    ) -> binary_core::BinaryResult<Self> {
        let kind = u64::read_from_section(cursor)?.to_le();
        let mode = (kind >> 24) as u8;
        #[deny(unreachable_patterns)]
        match mode {
            x if x == Self::Void.discriminant() => Ok(Self::Void),
            x if x == Self::U8.discriminant() => Ok(Self::U8),
            x if x == Self::I8.discriminant() => Ok(Self::I8),
            x if x == Self::U16.discriminant() => Ok(Self::U16),
            x if x == Self::I16.discriminant() => Ok(Self::I16),
            x if x == Self::U32.discriminant() => Ok(Self::U32),
            x if x == Self::I32.discriminant() => Ok(Self::I32),
            x if x == Self::U64.discriminant() => Ok(Self::U64),
            x if x == Self::I64.discriminant() => Ok(Self::I64),
            x if x == Self::String.discriminant() => Ok(Self::String),
            x if x == Self::Object.discriminant() => Ok(Self::Object),

            Self::STRUCTURE_DISCRIMINANT => {
                let len: u64 = kind & 0x00ffffff;
                let mut fields = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    fields.push(Self::read_from_section(cursor)?);
                }
                Ok(Self::Structure(fields))
            }
            _ => Err(binary_core::Error::EnumOutOfBounds(std::any::type_name::<
                Self,
            >())),
        }
    }
}

impl WriteToSection for NonPurusCallType {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> binary_core::BinaryResult<()> {
        let (discriminant, fields) = self.as_parts();
        let mut to_write_discriminant = (discriminant as u64) << 24;
        if let Some(fields) = fields {
            to_write_discriminant |= fields.len() as u64;
            to_write_discriminant.write_to_section(cursor)?;
            for f in fields {
                f.write_to_section(cursor)?;
            }
            Ok(())
        } else {
            to_write_discriminant.write_to_section(cursor)
        }
    }
}

#[inline(always)]
const fn match_size_signed<T>() -> NonPurusCallType {
    match size_of::<T>() {
        1 => NonPurusCallType::I8,
        2 => NonPurusCallType::I16,
        4 => NonPurusCallType::I32,
        8 => NonPurusCallType::I64,
        _ => panic!("Unsupported integer size"),
    }
}

#[inline(always)]
const fn match_size_unsigned<T>() -> NonPurusCallType {
    match size_of::<T>() {
        1 => NonPurusCallType::U8,
        2 => NonPurusCallType::U16,
        4 => NonPurusCallType::U32,
        8 => NonPurusCallType::U64,
        _ => panic!("Unsupported integer size"),
    }
}

const fn is_char_signed() -> bool {
    if c_char::MIN == 0 { false } else { true }
}

#[allow(nonstandard_style)]
// cSpell:disable
impl NonPurusCallType {
    pub const USize: Self = cfg_select! {
        target_pointer_width = "16" => { Self::U16 }
        target_pointer_width = "32" => { Self::U32 }
        target_pointer_width = "64" => { Self::U64 }
    };

    pub const ISize: Self = cfg_select! {
        target_pointer_width = "16" => { Self::I16 }
        target_pointer_width = "32" => { Self::I32 }
        target_pointer_width = "64" => { Self::I64 }
    };

    pub const Pointer: Self = Self::USize;

    pub const C_Char: Self = if is_char_signed() {
        match_size_signed::<c_char>()
    } else {
        match_size_unsigned::<c_char>()
    };
    pub const C_SChar: Self = match_size_signed::<c_schar>();
    pub const C_UChar: Self = match_size_unsigned::<c_uchar>();
}
// cSpell:enable

impl NonPurusCallType {
    pub fn layout(&self) -> Layout {
        match self {
            NonPurusCallType::Void => Layout::new::<()>(),
            NonPurusCallType::U8 => Layout::new::<u8>(),
            NonPurusCallType::I8 => Layout::new::<i8>(),
            NonPurusCallType::U16 => Layout::new::<u16>(),
            NonPurusCallType::I16 => Layout::new::<i16>(),
            NonPurusCallType::U32 => Layout::new::<u32>(),
            NonPurusCallType::I32 => Layout::new::<i32>(),
            NonPurusCallType::U64 => Layout::new::<u64>(),
            NonPurusCallType::I64 => Layout::new::<i64>(),
            NonPurusCallType::String => Layout::new::<NonNull<u8>>(),
            NonPurusCallType::Object => Layout::new::<NonNull<u8>>(),
            NonPurusCallType::Structure(types) => {
                let mut result = unsafe { Layout::from_size_align_unchecked(0, 1) };
                for ty in types {
                    (result, _) = result
                        .extend(ty.layout())
                        .ok()
                        .expect("Calculate Layout failed");
                }
                result
            }
        }
    }
}
