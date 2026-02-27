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
fn get_core_class(id: CoreTypeId, assembly: &Assembly) -> NonNull<Class> {
    *assembly.get_class(id as _).unwrap().unwrap()
}

#[inline(always)]
fn get_core_struct(id: CoreTypeId, assembly: &Assembly) -> NonNull<Struct> {
    *assembly.get_struct(id as _).unwrap().unwrap()
}

mod definitions;

pub use definitions::*;

pub trait CoreTypeIdExt: Sized {
    fn global_type_handle(self) -> NonGenericTypeHandle;
    /// Some type could not be passed in libffi and for those type it returns None
    fn val_libffi_type(self) -> Option<libffi::middle::Type>;
}

pub const trait CoreTypeIdConstExt: Sized {
    fn static_type_ref(self) -> TypeRef;
    fn mem_layout(self) -> Option<Layout>;
    fn val_layout(self) -> Option<Layout>;
    fn get_loader(self) -> fn(&Assembly) -> NonGenericTypeHandle;
}
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

            Self::System_NonPurusCallConfiguration => None,
            Self::System_NonPurusCallType => None,

            Self::System_DynamicLibrary => None,

            Self::System_Tuple => None,

            CoreTypeId::System_Array_1 => Some(Layout::new::<usize>()),
            CoreTypeId::System_String => Some(Layout::new::<u16>()),
            Self::System_LargeString => Some(Layout::new::<usize>()),

            Self::System_Environment => None,

            CoreTypeId::System_Exception => None,
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

            Self::System_NonPurusCallConfiguration => {
                Some(Layout::new::<ManagedReference<Class>>())
            }
            Self::System_NonPurusCallType => Some(Layout::new::<ManagedReference<Class>>()),
            Self::System_DynamicLibrary => Some(Layout::new::<ManagedReference<Class>>()),

            CoreTypeId::System_Tuple => None,

            CoreTypeId::System_Array_1 => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_String => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_LargeString => Some(Layout::new::<ManagedReference<Class>>()),

            CoreTypeId::System_Environment => Some(Layout::new::<ManagedReference<Class>>()),

            CoreTypeId::System_Exception => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_InvalidEnumException => {
                Some(Layout::new::<ManagedReference<Class>>())
            }
            CoreTypeId::System_Win32Exception => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_ErrnoException => Some(Layout::new::<ManagedReference<Class>>()),
            CoreTypeId::System_DlErrorException => Some(Layout::new::<ManagedReference<Class>>()),
        }
    }

    fn get_loader(self) -> fn(&Assembly) -> NonGenericTypeHandle {
        macro aider($($n:ident)*) {
            match self {
                $(
                    Self::$n => $n,
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

impl CoreTypeIdExt for CoreTypeId {
    fn global_type_handle(self) -> NonGenericTypeHandle {
        crate::virtual_machine::EnsureVirtualMachineInitialized();
        crate::virtual_machine::global_vm()
            .assembly_manager()
            .get_core_type(self)
    }

    fn val_libffi_type(self) -> Option<libffi::middle::Type> {
        use libffi::middle::Type;
        match self {
            Self::System_Object => Some(Type::pointer()),
            Self::System_ValueType => None,

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

            Self::System_NonPurusCallConfiguration => Some(Type::pointer()),
            Self::System_NonPurusCallType => Some(Type::pointer()),

            Self::System_DynamicLibrary => Some(Type::pointer()),

            Self::System_Tuple => None,

            Self::System_Array_1 => Some(Type::pointer()),
            Self::System_String => Some(Type::pointer()),
            Self::System_LargeString => Some(Type::pointer()),

            Self::System_Environment => Some(Type::pointer()),

            Self::System_Exception => Some(Type::pointer()),
            Self::System_InvalidEnumException => Some(Type::pointer()),
            Self::System_Win32Exception => Some(Type::pointer()),
            Self::System_ErrnoException => Some(Type::pointer()),
            Self::System_DlErrorException => Some(Type::pointer()),
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
                System_Object(assembly),
                System_ValueType(assembly),
                System_Void(assembly),
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
