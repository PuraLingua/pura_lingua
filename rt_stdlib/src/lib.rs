#![feature(decl_macro)]
#![allow(clippy::manual_non_exhaustive)]
#![allow(nonstandard_style)]

use global::{AllVariants, num_enum::TryFromPrimitive};

pub mod definitions;

#[repr(u32)]
#[derive(TryFromPrimitive, Clone, Copy, AllVariants, PartialEq, Eq)]
#[num_enum(crate = ::global::num_enum)]
pub enum CoreTypeId {
    System_Object,
    System_ValueType,

    System_Void,

    System_Nullable_1,

    /// 8 bits(just like [`bool`])
    System_Boolean,

    System_UInt8,
    System_UInt16,
    System_UInt32,
    System_UInt64,
    System_USize,

    System_Int8,
    System_Int16,
    System_Int32,
    System_Int64,
    System_ISize,

    /// It differs from rust's [`char`],
    /// which stores a Unicode scalar value.
    ///
    /// It stores a [`u16`]
    System_Char,

    System_Pointer,

    System_NonPurusCallConfiguration,
    System_NonPurusCallType,

    System_DynamicLibrary,

    System_Array_1,
    System_String,
    System_LargeString,

    System_Exception,
    System_InvalidEnumException,
    System_Win32Exception,
    System_ErrnoException,
    System_DlErrorException,
}

impl CoreTypeId {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::System_Object => "System::Object",
            Self::System_ValueType => "System::ValueType",
            Self::System_Void => "System::Void",

            Self::System_Nullable_1 => "System::Nullable`1",

            Self::System_Boolean => "System::Boolean",

            Self::System_UInt8 => "System::UInt8",
            Self::System_UInt16 => "System::UInt16",
            Self::System_UInt32 => "System::UInt32",
            Self::System_UInt64 => "System::UInt64",
            Self::System_USize => "System::USize",

            Self::System_Int8 => "System::Int8",
            Self::System_Int16 => "System::Int16",
            Self::System_Int32 => "System::Int32",
            Self::System_Int64 => "System::Int64",
            Self::System_ISize => "System::ISize",

            Self::System_Char => "System::Char",

            Self::System_Pointer => "System::Pointer",

            Self::System_NonPurusCallConfiguration => "System::NonPurusCallConfiguration",
            Self::System_NonPurusCallType => "System::NonPurusCallType",

            Self::System_DynamicLibrary => "System::DynamicLibrary",

            Self::System_Array_1 => "System::Array`1",

            Self::System_String => "System::String",
            Self::System_LargeString => "System::LargeString",

            Self::System_Exception => "System::Exception",
            Self::System_InvalidEnumException => "System::InvalidEnumException",
            Self::System_Win32Exception => "System::Win32Exception",
            Self::System_ErrnoException => "System::ErrnoException",
            Self::System_DlErrorException => "System::DlErrorException",
        }
    }
}

pub const CORE_ASSEMBLY_NAME: &str = "!";
