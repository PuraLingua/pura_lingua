#![allow(nonstandard_style, clippy::manual_non_exhaustive)]

use std::{alloc::Layout, ptr::NonNull};

use crate::{
    type_system::{
        assembly::{Assembly, TypeContainer},
        assembly_manager::{AssemblyManager, AssemblyRef},
        class::Class,
        r#struct::Struct,
        type_handle::NonGenericTypeHandle,
        type_ref::TypeRef,
    },
    value::managed_reference::ManagedReference,
};

mod System;

#[inline(always)]
#[allow(unused)]
fn get_core_class(id: CoreTypeId, assembly: &Assembly) -> NonNull<Class> {
    *assembly.get_class(id as _).unwrap().unwrap()
}

#[inline(always)]
#[allow(unused)]
fn get_core_struct(id: CoreTypeId, assembly: &Assembly) -> NonNull<Struct> {
    *assembly.get_struct(id as _).unwrap().unwrap()
}

pub trait CoreTypeIdExt: Sized {
    fn global_type_handle(self) -> NonGenericTypeHandle;
    /// Some types are not specified and for those types it returns None
    fn val_libffi_type(self) -> Option<libffi::middle::Type>;
    /// Some types are not specified and for those types it returns None
    fn non_purus_call_type(self) -> Option<NonPurusCallType>;
}

pub const trait CoreTypeIdConstExt: Sized {
    fn static_type_ref(self) -> TypeRef;
    fn mem_layout(self) -> Option<Layout>;
    fn val_layout(self) -> Option<Layout>;
    fn get_loader(self) -> fn(&Assembly) -> TypeContainer;
}
use global::non_purus_call_configuration::NonPurusCallType;
pub use stdlib_header::CoreTypeId;

impl const CoreTypeIdConstExt for CoreTypeId {
    fn static_type_ref(self) -> TypeRef {
        TypeRef::Index {
            assembly: AssemblyRef::CORE,
            ind: self as u32,
        }
    }

    fn mem_layout(self) -> Option<Layout> {
        const _: () = {
            #[allow(unused)]
            fn f()
            where
                global::assertions::LayoutEq<bool, u8>: global::assertions::SuccessAssert,
            {
            }
        };
        use CoreTypeId::*;

        match self {
            System_Object => Some(Layout::new::<()>()),
            System_ValueType => Some(Layout::new::<()>()),

            System_Void => Some(Layout::new::<()>()),

            System_Nullable_1 => None,

            System_Boolean => Some(Layout::new::<u8>()),

            System_UInt8 => Some(Layout::new::<u8>()),
            System_UInt16 => Some(Layout::new::<u16>()),
            System_UInt32 => Some(Layout::new::<u32>()),
            System_UInt64 => Some(Layout::new::<u64>()),
            System_USize => Some(Layout::new::<usize>()),

            System_Int8 => Some(Layout::new::<i8>()),
            System_Int16 => Some(Layout::new::<i16>()),
            System_Int32 => Some(Layout::new::<i32>()),
            System_Int64 => Some(Layout::new::<i64>()),
            System_ISize => Some(Layout::new::<isize>()),

            System_Char => Some(Layout::new::<u16>()),

            System_Pointer => Some(Layout::new::<*const u8>()),
            System_Reference_1 => Some(Layout::new::<*const u8>()),

            System_NonPurusCallConfiguration => None,
            System_NonPurusCallType => None,

            System_DynamicLibrary => None,

            System_IDispose => None,

            System_Tuple => None,

            System_Array_1 => Some(Layout::new::<usize>()),
            System_Span_1 => None,

            System_String => Some(Layout::new::<u16>()),
            System_LargeString => Some(Layout::new::<usize>()),

            System_RuntimeBasic => None,

            System_Exception
            | System_NullReferenceException
            | System_IndexOutOfRangeException
            | System_AllocException
            | System_InvalidEnumException
            | System_Win32Exception
            | System_ErrnoException
            | System_DlErrorException => None,

            System_Reflection_AssemblyInfo
            | System_Reflection_TypeInfo
            | System_Reflection_FieldInfo
            | System_Reflection_MethodInfo
            | System_Reflection_ParameterInfo => None,
        }
    }

    fn val_layout(self) -> Option<Layout> {
        use CoreTypeId::*;

        match self {
            System_Object => Some(Layout::new::<ManagedReference<Class>>()),
            System_ValueType => Some(Layout::new::<()>()),

            System_Void => Some(Layout::new::<()>()),

            // Even it's a struct, it has the same layout as an object
            System_Nullable_1 => Some(Layout::new::<ManagedReference<Class>>()),

            System_Boolean => Some(Layout::new::<u8>()),

            System_UInt8 => Some(Layout::new::<u8>()),
            System_UInt16 => Some(Layout::new::<u16>()),
            System_UInt32 => Some(Layout::new::<u32>()),
            System_UInt64 => Some(Layout::new::<u64>()),
            System_USize => Some(Layout::new::<usize>()),

            System_Int8 => Some(Layout::new::<i8>()),
            System_Int16 => Some(Layout::new::<i16>()),
            System_Int32 => Some(Layout::new::<i32>()),
            System_Int64 => Some(Layout::new::<i64>()),
            System_ISize => Some(Layout::new::<isize>()),

            System_Char => Some(Layout::new::<u16>()),

            System_Pointer => Some(Layout::new::<*const u8>()),
            System_Reference_1 => Some(Layout::new::<*const u8>()),

            System_NonPurusCallConfiguration => Some(Layout::new::<ManagedReference<Class>>()),
            System_NonPurusCallType => Some(Layout::new::<ManagedReference<Class>>()),

            System_DynamicLibrary => Some(Layout::new::<ManagedReference<Class>>()),

            System_IDispose => None,

            System_Tuple => None,

            System_Array_1 => Some(Layout::new::<ManagedReference<Class>>()),
            System_Span_1 => None,

            System_String => Some(Layout::new::<ManagedReference<Class>>()),
            System_LargeString => Some(Layout::new::<ManagedReference<Class>>()),

            System_RuntimeBasic => Some(Layout::new::<ManagedReference<Class>>()),

            System_Exception
            | System_NullReferenceException
            | System_IndexOutOfRangeException
            | System_AllocException
            | System_InvalidEnumException
            | System_Win32Exception
            | System_ErrnoException
            | System_DlErrorException => Some(Layout::new::<ManagedReference<Class>>()),

            System_Reflection_AssemblyInfo
            | System_Reflection_TypeInfo
            | System_Reflection_FieldInfo
            | System_Reflection_MethodInfo
            | System_Reflection_ParameterInfo => Some(Layout::new::<ManagedReference<Class>>()),
        }
    }

    fn get_loader(self) -> fn(&Assembly) -> TypeContainer {
        use CoreTypeId::*;

        macro of_System($name:ident) {
            System::$name::load
        }
        macro of_System_Reflection($name:ident) {
            System::Reflection::$name::load
        }
        match self {
            System_Object => of_System!(Object),
            System_ValueType => of_System!(ValueType),

            System_Void => of_System!(Void),

            System_Nullable_1 => of_System!(Nullable_1),

            System_Boolean => of_System!(Boolean),

            System_UInt8 => System::_Integers::System_UInt8,
            System_UInt16 => System::_Integers::System_UInt16,
            System_UInt32 => System::_Integers::System_UInt32,
            System_UInt64 => System::_Integers::System_UInt64,
            System_USize => System::_Integers::System_USize,

            System_Int8 => System::_Integers::System_Int8,
            System_Int16 => System::_Integers::System_Int16,
            System_Int32 => System::_Integers::System_Int32,
            System_Int64 => System::_Integers::System_Int64,
            System_ISize => System::_Integers::System_ISize,

            System_Char => of_System!(Char),

            System_Pointer => of_System!(Pointer),
            System_Reference_1 => of_System!(Reference_1),

            System_NonPurusCallConfiguration => of_System!(NonPurusCallConfiguration),
            System_NonPurusCallType => of_System!(NonPurusCallType),

            System_DynamicLibrary => of_System!(DynamicLibrary),

            System_IDispose => of_System!(IDispose),

            System_Tuple => of_System!(Tuple),

            System_Array_1 => of_System!(Array_1),
            System_Span_1 => of_System!(Span_1),

            System_String => of_System!(String),
            System_LargeString => of_System!(LargeString),

            System_RuntimeBasic => of_System!(RuntimeBasic),

            System_Exception => of_System!(Exception),
            System_NullReferenceException => of_System!(NullReferenceException),
            System_IndexOutOfRangeException => of_System!(IndexOutOfRangeException),
            System_AllocException => of_System!(AllocException),
            System_InvalidEnumException => of_System!(InvalidEnumException),
            System_Win32Exception => of_System!(Win32Exception),
            System_ErrnoException => of_System!(ErrnoException),
            System_DlErrorException => of_System!(DlErrorException),

            System_Reflection_AssemblyInfo => of_System_Reflection!(AssemblyInfo),
            System_Reflection_TypeInfo => of_System_Reflection!(TypeInfo),
            System_Reflection_FieldInfo => of_System_Reflection!(FieldInfo),
            System_Reflection_MethodInfo => of_System_Reflection!(MethodInfo),
            System_Reflection_ParameterInfo => of_System_Reflection!(ParameterInfo),
        }
    }
}

impl CoreTypeIdExt for CoreTypeId {
    fn global_type_handle(self) -> NonGenericTypeHandle {
        crate::virtual_machine::EnsureGlobalVirtualMachineInitialized();
        crate::virtual_machine::global_vm()
            .assembly_manager()
            .get_core_type(self)
    }

    fn val_libffi_type(self) -> Option<libffi::middle::Type> {
        use CoreTypeId::*;
        use libffi::middle::Type;

        match self {
            System_Object => Some(Type::pointer()),
            System_ValueType => Some(Type::void()),

            System_Void => Some(Type::void()),

            System_Nullable_1 => Some(Type::pointer()),

            System_Boolean => Some(Type::u8()),

            System_UInt8 => Some(Type::u8()),
            System_UInt16 => Some(Type::u16()),
            System_UInt32 => Some(Type::u32()),
            System_UInt64 => Some(Type::u64()),
            System_USize => Some(Type::usize()),

            System_Int8 => Some(Type::i8()),
            System_Int16 => Some(Type::i16()),
            System_Int32 => Some(Type::i32()),
            System_Int64 => Some(Type::i64()),
            System_ISize => Some(Type::isize()),

            System_Char => Some(Type::u16()),

            System_Pointer => Some(Type::pointer()),
            System_Reference_1 => Some(Type::pointer()),

            System_NonPurusCallConfiguration => Some(Type::pointer()),
            System_NonPurusCallType => Some(Type::pointer()),

            System_DynamicLibrary => Some(Type::pointer()),

            System_IDispose => None,

            System_Tuple => None,

            System_Array_1 => Some(Type::pointer()),
            System_Span_1 => Some(Type::pointer()),

            System_String => Some(Type::pointer()),
            System_LargeString => Some(Type::pointer()),

            System_RuntimeBasic => Some(Type::pointer()),

            System_Exception
            | System_NullReferenceException
            | System_IndexOutOfRangeException
            | System_AllocException
            | System_InvalidEnumException
            | System_Win32Exception
            | System_ErrnoException
            | System_DlErrorException => Some(Type::pointer()),

            System_Reflection_AssemblyInfo
            | System_Reflection_TypeInfo
            | System_Reflection_FieldInfo
            | System_Reflection_MethodInfo
            | System_Reflection_ParameterInfo => Some(Type::pointer()),
        }
    }
    fn non_purus_call_type(self) -> Option<NonPurusCallType> {
        use CoreTypeId::*;

        match self {
            System_Object => Some(NonPurusCallType::Object),
            System_ValueType => None,

            System_Void => Some(NonPurusCallType::Void),

            System_Nullable_1 => Some(NonPurusCallType::Object),

            System_Boolean => Some(NonPurusCallType::U8),

            System_UInt8 => Some(NonPurusCallType::U8),
            System_UInt16 => Some(NonPurusCallType::U16),
            System_UInt32 => Some(NonPurusCallType::U32),
            System_UInt64 => Some(NonPurusCallType::U64),
            System_USize => Some(NonPurusCallType::USize),

            System_Int8 => Some(NonPurusCallType::I8),
            System_Int16 => Some(NonPurusCallType::I16),
            System_Int32 => Some(NonPurusCallType::I32),
            System_Int64 => Some(NonPurusCallType::I64),
            System_ISize => Some(NonPurusCallType::ISize),

            System_Char => Some(NonPurusCallType::U16),

            System_Pointer => Some(NonPurusCallType::Pointer),
            System_Reference_1 => Some(NonPurusCallType::Pointer),

            System_NonPurusCallConfiguration => Some(NonPurusCallType::Object),
            System_NonPurusCallType => Some(NonPurusCallType::Object),

            System_DynamicLibrary => Some(NonPurusCallType::Object),

            System_IDispose => None,

            System_Tuple => None,

            System_Array_1 => Some(NonPurusCallType::Object),
            System_Span_1 => None,

            System_String => Some(NonPurusCallType::String),
            System_LargeString => Some(NonPurusCallType::Object),

            System_RuntimeBasic => Some(NonPurusCallType::Object),

            System_Exception
            | System_NullReferenceException
            | System_IndexOutOfRangeException
            | System_AllocException
            | System_InvalidEnumException
            | System_Win32Exception
            | System_ErrnoException
            | System_DlErrorException => Some(NonPurusCallType::Object),

            System_Reflection_AssemblyInfo
            | System_Reflection_TypeInfo
            | System_Reflection_FieldInfo
            | System_Reflection_MethodInfo
            | System_Reflection_ParameterInfo => Some(NonPurusCallType::Object),
        }
    }
}

pub fn load_stdlib(manager: &AssemblyManager) {
    let id = manager.add_assembly(Assembly::new_for_adding(
        stdlib_header::CORE_ASSEMBLY_NAME_W.to_owned(),
        true,
        |assembly| {
            let assembly = unsafe { assembly.as_ref() };
            vec![
                System::Object::load(assembly),
                System::ValueType::load(assembly),
                System::Void::load(assembly),
            ]
        },
    ));
    assert_eq!(AssemblyRef::CORE, id);

    let assembly = manager
        .get_assembly_by_ref(&AssemblyRef::CORE)
        .unwrap()
        .unwrap();

    for id in CoreTypeId::ALL_VARIANTS.into_iter().filter(|x| {
        !([
            CoreTypeId::System_Object,
            CoreTypeId::System_ValueType,
            CoreTypeId::System_Void,
        ]
        .contains(x))
    }) {
        let ind = assembly.add_type_handle(id.get_loader()(&assembly));
        debug_assert_eq!(ind, id as u32);
    }
}
