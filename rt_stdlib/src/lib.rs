#![feature(decl_macro)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(core_intrinsics)]
#![allow(clippy::manual_non_exhaustive)]
#![allow(nonstandard_style)]
#![allow(internal_features)]

use global::{AllVariants, AllVariantsName, num_enum::TryFromPrimitive};
use serde::{Deserialize, Serialize};

pub mod definitions;

#[repr(u32)]
#[derive(
    TryFromPrimitive,
    Clone,
    Copy,
    AllVariants,
    AllVariantsName,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
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

    System_Tuple,

    System_Array_1,
    System_String,
    System_LargeString,

    System_Environment,

    System_Exception,
    System_InvalidEnumException,
    System_Win32Exception,
    System_ErrnoException,
    System_DlErrorException,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreTypeRef {
    Core(CoreTypeId),
    WithGeneric(CoreTypeId, Vec<Self>),
    Generic(u32),
}

impl const From<CoreTypeId> for CoreTypeRef {
    #[inline(always)]
    fn from(value: CoreTypeId) -> Self {
        Self::Core(value)
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub enum CoreTypeKind {
    Class,
    Struct,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct GenericCount {
    pub count: u32,
    pub is_infinite: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreTypeInfo {
    pub id: CoreTypeId,
    pub kind: crate::CoreTypeKind,
    pub attr: global::attrs::TypeAttr,
    pub name: String,
    pub generic_count: Option<crate::GenericCount>,
    pub parent: Option<crate::CoreTypeRef>,
    pub parent_generics: Vec<CoreTypeRef>,
    pub methods: Vec<MethodInfo>,
    pub static_methods: Vec<MethodInfo>,
    pub fields: Vec<FieldInfo>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodInfo {
    pub id: u32,
    pub name: String,
    pub generic_count: Option<crate::GenericCount>,
    pub attr: global::attrs::MethodAttr<crate::CoreTypeRef>,
    pub args: Vec<(global::attrs::ParameterAttr, crate::CoreTypeRef)>,
    pub return_type: crate::CoreTypeRef,
}

impl MethodInfo {
    /// # Safety:
    /// [`Self::id`] must be created from `T`
    pub const unsafe fn get_id<T>(&self) -> T {
        const {
            assert!(size_of::<T>() == size_of::<u32>());
        }
        unsafe { std::intrinsics::transmute_unchecked(self.id) }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldInfo {
    pub id: u32,
    pub name: String,
    pub attr: global::attrs::FieldAttr,
    pub ty: crate::CoreTypeRef,
}

impl CoreTypeId {
    pub const fn raw_name(self) -> &'static str {
        Self::ALL_VARIANTS_NAME[self as u32 as usize]
    }
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

            Self::System_Tuple => "System::Tuple",

            Self::System_Array_1 => "System::Array`1",

            Self::System_String => "System::String",
            Self::System_LargeString => "System::LargeString",

            Self::System_Environment => "System::Environment",

            Self::System_Exception => "System::Exception",
            Self::System_InvalidEnumException => "System::InvalidEnumException",
            Self::System_Win32Exception => "System::Win32Exception",
            Self::System_ErrnoException => "System::ErrnoException",
            Self::System_DlErrorException => "System::DlErrorException",
        }
    }
}

impl CoreTypeId {
    pub fn get_core_type_info(self) -> fn() -> CoreTypeInfo {
        macro aider($($n:ident)*) {
            match self {
                $(
                    Self::$n => $crate::definitions::$n,
                )*
            }
        }

        aider!(
            System_Object
            System_ValueType

            System_Void

            System_Nullable_1

            System_Boolean

            System_UInt8
            System_UInt16
            System_UInt32
            System_UInt64
            System_USize

            System_Int8
            System_Int16
            System_Int32
            System_Int64
            System_ISize

            System_Char

            System_Pointer

            System_NonPurusCallConfiguration
            System_NonPurusCallType

            System_DynamicLibrary

            System_Tuple

            System_Array_1
            System_String
            System_LargeString

            System_Environment

            System_Exception
            System_InvalidEnumException
            System_Win32Exception
            System_ErrnoException
            System_DlErrorException
        )
    }
}

pub fn get_all_core_type_info() -> Vec<CoreTypeInfo> {
    let mut result = Vec::new();
    for x in CoreTypeId::ALL_VARIANTS {
        result.push(x.get_core_type_info()());
    }
    result
}

pub const CORE_ASSEMBLY_NAME: &str = "!";
