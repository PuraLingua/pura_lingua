#![feature(prelude_import)]
#![feature(decl_macro)]
#![allow(clippy::manual_non_exhaustive)]
#![allow(nonstandard_style)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use global::{AllVariants, num_enum::TryFromPrimitive};
pub mod definitions {
    #![allow(unused)]
    use std::ptr::NonNull;
    use global::attrs::CallConvention;
    use proc_macros::{define_core_class, define_core_struct, when_impl, when_not_impl};
    #[allow(unused)]
    use crate::CoreTypeId;
    #[repr(u32)]
    pub enum System_Object_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Object_MethodId {
        Destructor = 0u32,
        ToString = 1u32 + 0u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Object_StaticMethodId {
        StaticConstructor = System_Object_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_ValueType_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_ValueType_MethodId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_ValueType_StaticMethodId {
        StaticConstructor = System_ValueType_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_Void_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Void_MethodId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Void_StaticMethodId {
        StaticConstructor = System_Void_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_Nullable_1_FieldId {
        Inner,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Nullable_1_MethodId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Nullable_1_StaticMethodId {
        StaticConstructor = System_Nullable_1_MethodId::__END as u32,
        Initialize,
    }
    #[repr(u32)]
    pub enum System_Boolean_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Boolean_MethodId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Boolean_StaticMethodId {
        StaticConstructor = System_Boolean_MethodId::__END as u32,
    }
    mod integer {
        use proc_macros::{when_impl, when_not_impl};
        use crate::CoreTypeId;
        #[repr(u32)]
        pub enum System_UInt8_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt8_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt8_StaticMethodId {
            StaticConstructor = System_UInt8_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_UInt16_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt16_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt16_StaticMethodId {
            StaticConstructor = System_UInt16_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_UInt32_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt32_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt32_StaticMethodId {
            StaticConstructor = System_UInt32_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_UInt64_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt64_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_UInt64_StaticMethodId {
            StaticConstructor = System_UInt64_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_USize_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_USize_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_USize_StaticMethodId {
            StaticConstructor = System_USize_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_Int8_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int8_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int8_StaticMethodId {
            StaticConstructor = System_Int8_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_Int16_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int16_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int16_StaticMethodId {
            StaticConstructor = System_Int16_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_Int32_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int32_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int32_StaticMethodId {
            StaticConstructor = System_Int32_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_Int64_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int64_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_Int64_StaticMethodId {
            StaticConstructor = System_Int64_MethodId::__END as u32,
            ToString,
        }
        #[repr(u32)]
        pub enum System_ISize_FieldId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_ISize_MethodId {
            #[doc(hidden)]
            __END,
        }
        #[repr(u32)]
        pub enum System_ISize_StaticMethodId {
            StaticConstructor = System_ISize_MethodId::__END as u32,
            ToString,
        }
    }
    pub use integer::*;
    #[repr(u32)]
    pub enum System_Char_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Char_MethodId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Char_StaticMethodId {
        StaticConstructor = System_Char_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_Pointer_FieldId {
        Null,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Pointer_MethodId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Pointer_StaticMethodId {
        StaticConstructor = System_Pointer_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_NonPurusCallConfiguration_FieldId {
        CallConvention = 0u32,
        ReturnType = 1u32 + 0u32,
        Encoding = 1u32 + 1u32 + 0u32,
        ObjectStrategy = 1u32 + 1u32 + 1u32 + 0u32,
        ByRefArguments = 1u32 + 1u32 + 1u32 + 1u32 + 0u32,
        Arguments = 1u32 + 1u32 + 1u32 + 1u32 + 1u32 + 0u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_NonPurusCallConfiguration_MethodId {
        Constructor = System_Object_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_NonPurusCallConfiguration_StaticMethodId {
        StaticConstructor = System_NonPurusCallConfiguration_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_NonPurusCallType_FieldId {
        Discriminant = System_Object_FieldId::__END as u32,
        Types = 1u32 + System_Object_FieldId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_NonPurusCallType_MethodId {
        #[doc(hidden)]
        __END = System_Object_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_NonPurusCallType_StaticMethodId {
        StaticConstructor = System_NonPurusCallType_MethodId::__END as u32,
        CreateVoid,
        CreateU8,
        CreateI8,
        CreateU16,
        CreateI16,
        CreateU32,
        CreateI32,
        CreateU64,
        CreateI64,
        CreatePointer,
        CreateString,
        CreateObject,
        CreateStructure,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_DynamicLibrary_FieldId {
        Handle = System_Object_FieldId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_DynamicLibrary_MethodId {
        Destructor = (System_Object_MethodId::Destructor as u32),
        Constructor_String = System_Object_MethodId::__END as u32,
        GetSymbol = 1u32 + System_Object_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_DynamicLibrary_StaticMethodId {
        StaticConstructor = System_DynamicLibrary_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Array_1_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Array_1_MethodId {
        Destructor = (System_Object_MethodId::Destructor as u32),
        ToString = (System_Object_MethodId::ToString as u32),
        GetPointerOfIndex = System_Object_MethodId::__END as u32,
        get_Index = 1u32 + System_Object_MethodId::__END as u32,
        set_Index = 1u32 + 1u32 + System_Object_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Array_1_StaticMethodId {
        StaticConstructor = System_Array_1_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_String_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_String_MethodId {
        ToString = (System_Object_MethodId::ToString as u32),
        #[doc(hidden)]
        __END = System_Object_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_String_StaticMethodId {
        StaticConstructor = System_String_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_LargeString_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_LargeString_MethodId {
        ToString = (System_Object_MethodId::ToString as u32),
        #[doc(hidden)]
        __END = System_Object_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_LargeString_StaticMethodId {
        StaticConstructor = System_LargeString_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Exception_FieldId {
        Message = System_Object_FieldId::__END as u32,
        Inner = 1u32 + System_Object_FieldId::__END as u32,
        StackTrace = 1u32 + 1u32 + System_Object_FieldId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Exception_MethodId {
        ToString = (System_Object_MethodId::ToString as u32),
        Constructor_String = System_Object_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Exception_StaticMethodId {
        StaticConstructor = System_Exception_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_InvalidEnumException_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_InvalidEnumException_MethodId {
        Constructor_String_String = System_Exception_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_InvalidEnumException_StaticMethodId {
        StaticConstructor = System_InvalidEnumException_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Win32Exception_FieldId {
        Code = System_Object_FieldId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Win32Exception_MethodId {
        Constructor_I32 = System_Exception_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_Win32Exception_StaticMethodId {
        StaticConstructor = System_Win32Exception_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_ErrnoException_FieldId {
        Code = System_Object_FieldId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_ErrnoException_MethodId {
        Constructor_I32 = System_Exception_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_ErrnoException_StaticMethodId {
        StaticConstructor = System_ErrnoException_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_DlErrorException_FieldId {
        #[doc(hidden)]
        __END,
    }
    #[repr(u32)]
    pub enum System_DlErrorException_MethodId {
        #[doc(hidden)]
        __END = System_Exception_MethodId::__END as u32,
    }
    #[repr(u32)]
    pub enum System_DlErrorException_StaticMethodId {
        StaticConstructor = System_DlErrorException_MethodId::__END as u32,
        #[doc(hidden)]
        __END,
    }
}
#[repr(u32)]
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
impl ::global::num_enum::TryFromPrimitive for CoreTypeId {
    type Primitive = u32;
    type Error = ::global::num_enum::TryFromPrimitiveError<Self>;
    const NAME: &'static str = "CoreTypeId";
    fn try_from_primitive(
        number: Self::Primitive,
    ) -> ::core::result::Result<Self, ::global::num_enum::TryFromPrimitiveError<Self>> {
        #![allow(non_upper_case_globals)]
        const System_Object__num_enum_0__: u32 = 0;
        const System_ValueType__num_enum_0__: u32 = 1;
        const System_Void__num_enum_0__: u32 = 2;
        const System_Nullable_1__num_enum_0__: u32 = 3;
        const System_Boolean__num_enum_0__: u32 = 4;
        const System_UInt8__num_enum_0__: u32 = 5;
        const System_UInt16__num_enum_0__: u32 = 6;
        const System_UInt32__num_enum_0__: u32 = 7;
        const System_UInt64__num_enum_0__: u32 = 8;
        const System_USize__num_enum_0__: u32 = 9;
        const System_Int8__num_enum_0__: u32 = 10;
        const System_Int16__num_enum_0__: u32 = 11;
        const System_Int32__num_enum_0__: u32 = 12;
        const System_Int64__num_enum_0__: u32 = 13;
        const System_ISize__num_enum_0__: u32 = 14;
        const System_Char__num_enum_0__: u32 = 15;
        const System_Pointer__num_enum_0__: u32 = 16;
        const System_NonPurusCallConfiguration__num_enum_0__: u32 = 17;
        const System_NonPurusCallType__num_enum_0__: u32 = 18;
        const System_DynamicLibrary__num_enum_0__: u32 = 19;
        const System_Array_1__num_enum_0__: u32 = 20;
        const System_String__num_enum_0__: u32 = 21;
        const System_LargeString__num_enum_0__: u32 = 22;
        const System_Exception__num_enum_0__: u32 = 23;
        const System_InvalidEnumException__num_enum_0__: u32 = 24;
        const System_Win32Exception__num_enum_0__: u32 = 25;
        const System_ErrnoException__num_enum_0__: u32 = 26;
        const System_DlErrorException__num_enum_0__: u32 = 27;
        #[deny(unreachable_patterns)]
        match number {
            System_Object__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_Object)
            }
            System_ValueType__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_ValueType)
            }
            System_Void__num_enum_0__ => ::core::result::Result::Ok(Self::System_Void),
            System_Nullable_1__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_Nullable_1)
            }
            System_Boolean__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_Boolean)
            }
            System_UInt8__num_enum_0__ => ::core::result::Result::Ok(Self::System_UInt8),
            System_UInt16__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_UInt16)
            }
            System_UInt32__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_UInt32)
            }
            System_UInt64__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_UInt64)
            }
            System_USize__num_enum_0__ => ::core::result::Result::Ok(Self::System_USize),
            System_Int8__num_enum_0__ => ::core::result::Result::Ok(Self::System_Int8),
            System_Int16__num_enum_0__ => ::core::result::Result::Ok(Self::System_Int16),
            System_Int32__num_enum_0__ => ::core::result::Result::Ok(Self::System_Int32),
            System_Int64__num_enum_0__ => ::core::result::Result::Ok(Self::System_Int64),
            System_ISize__num_enum_0__ => ::core::result::Result::Ok(Self::System_ISize),
            System_Char__num_enum_0__ => ::core::result::Result::Ok(Self::System_Char),
            System_Pointer__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_Pointer)
            }
            System_NonPurusCallConfiguration__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_NonPurusCallConfiguration)
            }
            System_NonPurusCallType__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_NonPurusCallType)
            }
            System_DynamicLibrary__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_DynamicLibrary)
            }
            System_Array_1__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_Array_1)
            }
            System_String__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_String)
            }
            System_LargeString__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_LargeString)
            }
            System_Exception__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_Exception)
            }
            System_InvalidEnumException__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_InvalidEnumException)
            }
            System_Win32Exception__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_Win32Exception)
            }
            System_ErrnoException__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_ErrnoException)
            }
            System_DlErrorException__num_enum_0__ => {
                ::core::result::Result::Ok(Self::System_DlErrorException)
            }
            #[allow(unreachable_patterns)]
            _ => {
                ::core::result::Result::Err(
                    ::global::num_enum::TryFromPrimitiveError::<Self>::new(number),
                )
            }
        }
    }
}
impl ::core::convert::TryFrom<u32> for CoreTypeId {
    type Error = ::global::num_enum::TryFromPrimitiveError<Self>;
    #[inline]
    fn try_from(
        number: u32,
    ) -> ::core::result::Result<Self, ::global::num_enum::TryFromPrimitiveError<Self>> {
        ::global::num_enum::TryFromPrimitive::try_from_primitive(number)
    }
}
#[doc(hidden)]
impl ::global::num_enum::CannotDeriveBothFromPrimitiveAndTryFromPrimitive
for CoreTypeId {}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for CoreTypeId {}
#[automatically_derived]
impl ::core::clone::Clone for CoreTypeId {
    #[inline]
    fn clone(&self) -> CoreTypeId {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for CoreTypeId {}
impl CoreTypeId {
    pub const ALL_VARIANTS: [Self; 28usize] = [
        Self::System_Object,
        Self::System_ValueType,
        Self::System_Void,
        Self::System_Nullable_1,
        Self::System_Boolean,
        Self::System_UInt8,
        Self::System_UInt16,
        Self::System_UInt32,
        Self::System_UInt64,
        Self::System_USize,
        Self::System_Int8,
        Self::System_Int16,
        Self::System_Int32,
        Self::System_Int64,
        Self::System_ISize,
        Self::System_Char,
        Self::System_Pointer,
        Self::System_NonPurusCallConfiguration,
        Self::System_NonPurusCallType,
        Self::System_DynamicLibrary,
        Self::System_Array_1,
        Self::System_String,
        Self::System_LargeString,
        Self::System_Exception,
        Self::System_InvalidEnumException,
        Self::System_Win32Exception,
        Self::System_ErrnoException,
        Self::System_DlErrorException,
    ];
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for CoreTypeId {}
#[automatically_derived]
impl ::core::cmp::PartialEq for CoreTypeId {
    #[inline]
    fn eq(&self, other: &CoreTypeId) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for CoreTypeId {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) {}
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
