#![allow(nonstandard_style, clippy::manual_non_exhaustive)]

use std::{alloc::Layout, ptr::NonNull};

use crate::{
    type_system::{
        assembly::Assembly,
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
    fn get_loader(self) -> fn(&Assembly) -> NonGenericTypeHandle;
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
        match self {
            CoreTypeId::System_Object => Some(Layout::new::<()>()),
            CoreTypeId::System_ValueType => Some(Layout::new::<()>()),

            CoreTypeId::System_Void => Some(Layout::new::<()>()),

            CoreTypeId::System_Nullable_1 => None,

            Self::System_Boolean => Some(Layout::new::<u8>()),

            CoreTypeId::System_UInt8 => Some(Layout::new::<u8>()),
            CoreTypeId::System_UInt16 => Some(Layout::new::<u16>()),
            CoreTypeId::System_UInt32 => Some(Layout::new::<u32>()),
            CoreTypeId::System_UInt64 => Some(Layout::new::<u64>()),
            CoreTypeId::System_USize => Some(Layout::new::<usize>()),

            CoreTypeId::System_Int8 => Some(Layout::new::<i8>()),
            CoreTypeId::System_Int16 => Some(Layout::new::<i16>()),
            CoreTypeId::System_Int32 => Some(Layout::new::<i32>()),
            CoreTypeId::System_Int64 => Some(Layout::new::<i64>()),
            CoreTypeId::System_ISize => Some(Layout::new::<isize>()),

            CoreTypeId::System_Char => Some(Layout::new::<u16>()),

            Self::System_Pointer => Some(Layout::new::<*const u8>()),
            Self::System_Reference_1 => Some(Layout::new::<*const u8>()),

            Self::System_NonPurusCallConfiguration => None,
            Self::System_NonPurusCallType => None,

            Self::System_DynamicLibrary => None,

            Self::System_Tuple => None,

            CoreTypeId::System_Array_1 => Some(Layout::new::<usize>()),
            Self::System_Span_1 => None,

            CoreTypeId::System_String => Some(Layout::new::<u16>()),
            Self::System_LargeString => Some(Layout::new::<usize>()),

            Self::System_RuntimeBasic => None,

            CoreTypeId::System_Exception => None,
            CoreTypeId::System_AllocException => None,
            CoreTypeId::System_InvalidEnumException => None,
            Self::System_Win32Exception => None,
            Self::System_ErrnoException => None,
            Self::System_DlErrorException => None,
        }
    }

    fn val_layout(self) -> Option<Layout> {
        match self {
            CoreTypeId::System_Object => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_ValueType => Some(Layout::new::<()>()),

            CoreTypeId::System_Void => Some(Layout::new::<()>()),

            // Even it's a struct, it has the same layout as an object
            CoreTypeId::System_Nullable_1 => Some(Layout::new::<ManagedReference<Class>>()),

            Self::System_Boolean => Some(Layout::new::<u8>()),

            CoreTypeId::System_UInt8 => Some(Layout::new::<u8>()),
            CoreTypeId::System_UInt16 => Some(Layout::new::<u16>()),
            CoreTypeId::System_UInt32 => Some(Layout::new::<u32>()),
            CoreTypeId::System_UInt64 => Some(Layout::new::<u64>()),
            CoreTypeId::System_USize => Some(Layout::new::<usize>()),

            CoreTypeId::System_Int8 => Some(Layout::new::<i8>()),
            CoreTypeId::System_Int16 => Some(Layout::new::<i16>()),
            CoreTypeId::System_Int32 => Some(Layout::new::<i32>()),
            CoreTypeId::System_Int64 => Some(Layout::new::<i64>()),
            CoreTypeId::System_ISize => Some(Layout::new::<isize>()),

            CoreTypeId::System_Char => Some(Layout::new::<u16>()),

            Self::System_Pointer => Some(Layout::new::<*const u8>()),
            Self::System_Reference_1 => Some(Layout::new::<*const u8>()),

            Self::System_NonPurusCallConfiguration => {
                Some(Layout::new::<ManagedReference<Class>>())
            }
            Self::System_NonPurusCallType => Some(Layout::new::<ManagedReference<Class>>()),
            Self::System_DynamicLibrary => Some(Layout::new::<ManagedReference<Class>>()),

            CoreTypeId::System_Tuple => None,

            CoreTypeId::System_Array_1 => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_Span_1 => None,

            CoreTypeId::System_String => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_LargeString => Some(Layout::new::<ManagedReference<Class>>()),

            CoreTypeId::System_RuntimeBasic => Some(Layout::new::<ManagedReference<Class>>()),

            CoreTypeId::System_Exception
            | CoreTypeId::System_AllocException
            | CoreTypeId::System_InvalidEnumException
            | CoreTypeId::System_Win32Exception
            | CoreTypeId::System_ErrnoException
            | CoreTypeId::System_DlErrorException => Some(Layout::new::<ManagedReference<Class>>()),
        }
    }

    fn get_loader(self) -> fn(&Assembly) -> NonGenericTypeHandle {
        macro of($name:ident) {
            System::$name::load
        }
        match self {
            Self::System_Object => of!(Object),
            Self::System_ValueType => of!(ValueType),

            Self::System_Void => of!(Void),

            Self::System_Nullable_1 => of!(Nullable_1),

            Self::System_Boolean => of!(Boolean),

            Self::System_UInt8 => System::_Integers::System_UInt8,
            Self::System_UInt16 => System::_Integers::System_UInt16,
            Self::System_UInt32 => System::_Integers::System_UInt32,
            Self::System_UInt64 => System::_Integers::System_UInt64,
            Self::System_USize => System::_Integers::System_USize,

            Self::System_Int8 => System::_Integers::System_Int8,
            Self::System_Int16 => System::_Integers::System_Int16,
            Self::System_Int32 => System::_Integers::System_Int32,
            Self::System_Int64 => System::_Integers::System_Int64,
            Self::System_ISize => System::_Integers::System_ISize,

            Self::System_Char => of!(Char),

            Self::System_Pointer => of!(Pointer),
            Self::System_Reference_1 => of!(Reference_1),

            Self::System_NonPurusCallConfiguration => of!(NonPurusCallConfiguration),
            Self::System_NonPurusCallType => of!(NonPurusCallType),

            Self::System_DynamicLibrary => of!(DynamicLibrary),

            Self::System_Tuple => of!(Tuple),

            Self::System_Array_1 => of!(Array_1),
            Self::System_Span_1 => of!(Span_1),

            Self::System_String => of!(String),
            Self::System_LargeString => of!(LargeString),

            Self::System_RuntimeBasic => of!(RuntimeBasic),

            Self::System_Exception => of!(Exception),
            Self::System_AllocException => of!(AllocException),
            Self::System_InvalidEnumException => of!(InvalidEnumException),
            Self::System_Win32Exception => of!(Win32Exception),
            Self::System_ErrnoException => of!(ErrnoException),
            Self::System_DlErrorException => of!(DlErrorException),
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
        use libffi::middle::Type;
        match self {
            Self::System_Object => Some(Type::pointer()),
            Self::System_ValueType => Some(Type::void()),

            Self::System_Void => Some(Type::void()),

            Self::System_Nullable_1 => Some(Type::pointer()),

            Self::System_Boolean => Some(Type::u8()),

            Self::System_UInt8 => Some(Type::u8()),
            Self::System_UInt16 => Some(Type::u16()),
            Self::System_UInt32 => Some(Type::u32()),
            Self::System_UInt64 => Some(Type::u64()),
            Self::System_USize => Some(Type::usize()),

            Self::System_Int8 => Some(Type::i8()),
            Self::System_Int16 => Some(Type::i16()),
            Self::System_Int32 => Some(Type::i32()),
            Self::System_Int64 => Some(Type::i64()),
            Self::System_ISize => Some(Type::isize()),

            Self::System_Char => Some(Type::u16()),

            Self::System_Pointer => Some(Type::pointer()),
            Self::System_Reference_1 => Some(Type::pointer()),

            Self::System_NonPurusCallConfiguration => Some(Type::pointer()),
            Self::System_NonPurusCallType => Some(Type::pointer()),

            Self::System_DynamicLibrary => Some(Type::pointer()),

            Self::System_Tuple => None,

            Self::System_Array_1 => Some(Type::pointer()),
            Self::System_Span_1 => Some(Type::pointer()),

            Self::System_String => Some(Type::pointer()),
            Self::System_LargeString => Some(Type::pointer()),

            Self::System_RuntimeBasic => Some(Type::pointer()),

            Self::System_Exception
            | Self::System_AllocException
            | Self::System_InvalidEnumException
            | Self::System_Win32Exception
            | Self::System_ErrnoException
            | Self::System_DlErrorException => Some(Type::pointer()),
        }
    }
    fn non_purus_call_type(self) -> Option<NonPurusCallType> {
        match self {
            Self::System_Object => Some(NonPurusCallType::Object),
            Self::System_ValueType => None,

            Self::System_Void => Some(NonPurusCallType::Void),

            Self::System_Nullable_1 => Some(NonPurusCallType::Object),

            Self::System_Boolean => Some(NonPurusCallType::U8),

            Self::System_UInt8 => Some(NonPurusCallType::U8),
            Self::System_UInt16 => Some(NonPurusCallType::U16),
            Self::System_UInt32 => Some(NonPurusCallType::U32),
            Self::System_UInt64 => Some(NonPurusCallType::U64),
            Self::System_USize => Some(NonPurusCallType::USize),

            Self::System_Int8 => Some(NonPurusCallType::I8),
            Self::System_Int16 => Some(NonPurusCallType::I16),
            Self::System_Int32 => Some(NonPurusCallType::I32),
            Self::System_Int64 => Some(NonPurusCallType::I64),
            Self::System_ISize => Some(NonPurusCallType::ISize),

            Self::System_Char => Some(NonPurusCallType::U16),

            Self::System_Pointer => Some(NonPurusCallType::Pointer),
            Self::System_Reference_1 => Some(NonPurusCallType::Pointer),

            Self::System_NonPurusCallConfiguration => Some(NonPurusCallType::Object),
            Self::System_NonPurusCallType => Some(NonPurusCallType::Object),

            Self::System_DynamicLibrary => Some(NonPurusCallType::Object),

            Self::System_Tuple => None,

            Self::System_Array_1 => Some(NonPurusCallType::Object),
            Self::System_Span_1 => None,

            Self::System_String => Some(NonPurusCallType::String),
            Self::System_LargeString => Some(NonPurusCallType::Object),

            Self::System_RuntimeBasic => Some(NonPurusCallType::Object),

            Self::System_Exception
            | Self::System_AllocException
            | Self::System_InvalidEnumException
            | Self::System_Win32Exception
            | Self::System_ErrnoException
            | Self::System_DlErrorException => Some(NonPurusCallType::Object),
        }
    }
}

pub fn load_stdlib(manager: &AssemblyManager) {
    let id = manager.add_assembly(Assembly::new_for_adding(
        stdlib_header::CORE_ASSEMBLY_NAME.to_owned(),
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
