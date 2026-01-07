#![allow(nonstandard_style, clippy::manual_non_exhaustive)]

use std::{alloc::Layout, ptr::NonNull};

use global::{attrs::CallConvention, num_enum::TryFromPrimitive, string_name};
use proc_macros::{define_core_class, define_core_struct};

use crate::{
    type_system::{
        assembly::Assembly,
        assembly_manager::AssemblyManager,
        class::Class,
        method::{Method, Parameter},
        r#struct::Struct,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
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

define_core_class! {
    #[Public {}] assembly
    System_Object "System.Object" =>
    #fields:

    #methods:
    [Destructor ToString] [] with
    |mt| vec![
        Box::new(Method::native(
            Some(mt),
            "~".to_owned(),
            global::attr!(
                method Public {}
            ),
            vec![],
            MaybeUnloadedTypeHandle::Unloaded(
                CoreTypeId::System_Void.static_type_ref(),
            ),
            CallConvention::PlatformDefault,
            None,
            System::Object::Destructor as _,
        )),
        Box::new(Method::native(
            Some(mt),
            "ToString()".to_owned(),
            global::attr!(
                method Public {}
            ),
            vec![],
            MaybeUnloadedTypeHandle::Unloaded(
                CoreTypeId::System_String.static_type_ref(),
            ),
            CallConvention::PlatformDefault,
            None,
            System::Object::ToString as _,
        )),

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

define_core_struct! {
    #[Public {}] assembly
    System_ValueType "System.ValueType" =>
    #fields:

    #methods:
    [] [] with
    |mt| vec![

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

define_core_struct! {
    #[Public {}] assembly
    System_Void "System.Void" =>
    #fields:

    #methods:
    [] [] with
    |mt| vec![

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

macro define_ints(
    $(
        $RustT:ident => $loader:ident $name:literal;
    )*
    #methods:
	$ToString:ident @with
    $CurrentRustT:ident $CurrentId:ident
	$method_generator:expr
) {$(
	proc_macros::define_core_struct! {
		#[Public {}] assembly
		$loader $name =>
		[None]
		#fields:
		#methods:
		[] [$ToString] with
		{
            type $CurrentRustT = $RustT;
            const $CurrentId: CoreTypeId = CoreTypeId::$loader;
            fn _a() where $CurrentRustT: 'static {}
            $method_generator
        }
	}
)*}

define_ints! {
    u8 => System_UInt8 "System.UInt8";
    u16 => System_UInt16 "System.UInt16";
    u32 => System_UInt32 "System.UInt32";
    u64 => System_UInt64 "System.UInt64";
    usize => System_USize "System.USize";

    i8 => System_Int8 "System.Int8";
    i16 => System_Int16 "System.Int16";
    i32 => System_Int32 "System.Int32";
    i64 => System_Int64 "System.Int64";
    isize => System_ISize "System.ISize";

    #methods:
    ToString @with
    RustT CORE_ID
    |mt| {
        use crate::type_system::method::{Method, Parameter};

        vec![
            // Statics
            Box::new(
                Method::default_sctor(
                    Some(mt),
                    global::attr!(
                        method Public {Static}
                    ),
                ),
            ),
            Box::new(Method::native(
                Some(mt),
                "ToString()".to_owned(),
                global::attr!(
                    method Public {Static}
                ),
                vec![
                    Parameter::new(
                        MaybeUnloadedTypeHandle::Unloaded(CORE_ID.static_type_ref()),
                        global::attr!(
                            parameter {ByRef}
                        ),
                    )
                ],
                MaybeUnloadedTypeHandle::Unloaded(
                    CoreTypeId::System_String.static_type_ref(),
                ),
                global::attrs::CallConvention::PlatformDefault,
                None,
                System::_Integers::ToString::<RustT> as _,
            )),

        ]
    }
}

define_core_struct! {
    #[Public {}] assembly
    System_Char "System.Char" =>
    #fields:

    #methods:
    [] [] with
    |mt| vec![

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_Array_1 "System.Array`1" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields:

    #methods of System_Object_MethodId:
    [override ToString] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                "ToString()".to_owned(),
                global::attr!(
                    method override Some(System_Object_MethodId::ToString as _) Public {}
                ),
                vec![],
                MaybeUnloadedTypeHandle::Unloaded(
                    CoreTypeId::System_String.static_type_ref(),
                ),
                CallConvention::PlatformDefault,
                None,
                System::Array::ToString as _,
            ),
        ),
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_String "System.String" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields:

    #methods of System_Object_MethodId:
    [override ToString] [] with
    |mt| vec![
        Box::new(Method::native(
            Some(mt),
            "ToString()".to_owned(),
            global::attr!(
                method override Some(System_Object_MethodId::ToString as _) Public {}
            ),
            vec![],
            MaybeUnloadedTypeHandle::Unloaded(
                CoreTypeId::System_String.static_type_ref(),
            ),
            CallConvention::PlatformDefault,
            None,
            System::String::ToString as _,
        )),

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_Exception "System.Exception" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields:
    #[Public {}] Message "_message" => CoreTypeId::System_String.static_type_ref().into();
    #[Public {}] Inner "_innerException" => CoreTypeId::System_Exception.static_type_ref().into();
    #[Public {}] StackTrace "_stackTrace" => TypeRef::Specific {
        assembly: string_name!("!"),
        ind: CoreTypeId::System_Array_1 as _,
        types: vec![
            CoreTypeId::System_String.static_type_ref().into(),
        ],
    }.into();

    #methods of System_Object_MethodId:
    [override ToString Constructor_String] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                "ToString()".to_owned(),
                global::attr!(
                    method override Some(System_Object_MethodId::ToString as _) Public {}
                ),
                vec![],
                get_core_class(CoreTypeId::System_String, assembly).into(),
                CallConvention::PlatformDefault,
                None,
                System::Exception::ToString as _,
            ),
        ),
        Box::new(
            Method::native(
                Some(mt),
                ".ctor".to_owned(),
                global::attr!(
                    method Public {HideWhenCapturing}
                ),
                vec![
                    Parameter::new(
                        CoreTypeId::System_String.static_type_ref().into(),
                        global::attr!(
                            parameter {}
                        ),
                    ),
                ],
                get_core_struct(CoreTypeId::System_Void, assembly).into(),
                CallConvention::PlatformDefault,
                None,
                System::Exception::Construct_String as _,
            )
        ),
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

#[repr(u32)]
#[derive(TryFromPrimitive, Clone, Copy, PartialEq, Eq)]
#[num_enum(crate = ::global::num_enum)]
pub enum CoreTypeId {
    System_Object,
    System_ValueType,

    System_Void,

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
    /// It stores a u16
    System_Char,

    System_Array_1,
    System_String,
    System_Exception,
}

impl CoreTypeId {
    pub fn global_type_handle(self) -> NonGenericTypeHandle {
        crate::virtual_machine::EnsureVirtualMachineInitialized();
        crate::virtual_machine::global_vm()
            .assembly_manager()
            .get_core_type(self)
    }

    pub const fn static_type_ref(self) -> TypeRef {
        TypeRef::Index {
            assembly: string_name!("!"),
            ind: self as u32,
        }
    }

    pub const fn mem_layout(self) -> Option<Layout> {
        match self {
            CoreTypeId::System_Object => Some(Layout::new::<()>()),
            CoreTypeId::System_ValueType => Some(Layout::new::<()>()),

            CoreTypeId::System_Void => Some(Layout::new::<()>()),

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

            CoreTypeId::System_Array_1 => Some(Layout::new::<usize>()),
            CoreTypeId::System_String => Some(Layout::new::<usize>()),
            CoreTypeId::System_Exception => None,
        }
    }

    pub const fn val_layout(self) -> Layout {
        match self {
            CoreTypeId::System_Object => Layout::new::<ManagedReference<Class>>(),
            CoreTypeId::System_ValueType => Layout::new::<()>(),

            CoreTypeId::System_Void => Layout::new::<()>(),

            CoreTypeId::System_UInt8 => Layout::new::<u8>(),
            CoreTypeId::System_UInt16 => Layout::new::<u16>(),
            CoreTypeId::System_UInt32 => Layout::new::<u32>(),
            CoreTypeId::System_UInt64 => Layout::new::<u64>(),
            CoreTypeId::System_USize => Layout::new::<usize>(),

            CoreTypeId::System_Int8 => Layout::new::<i8>(),
            CoreTypeId::System_Int16 => Layout::new::<i16>(),
            CoreTypeId::System_Int32 => Layout::new::<i32>(),
            CoreTypeId::System_Int64 => Layout::new::<i64>(),
            CoreTypeId::System_ISize => Layout::new::<isize>(),

            CoreTypeId::System_Char => Layout::new::<u16>(),

            CoreTypeId::System_Array_1 => Layout::new::<ManagedReference<Class>>(),
            CoreTypeId::System_String => Layout::new::<ManagedReference<Class>>(),
            CoreTypeId::System_Exception => Layout::new::<ManagedReference<Class>>(),
        }
    }

    /// Some type could not be passed in libffi and for those type it returns None
    pub fn val_libffi_type(self) -> Option<libffi::middle::Type> {
        use libffi::middle::Type;
        match self {
            Self::System_Object => Some(Type::pointer()),
            Self::System_ValueType => None,

            Self::System_Void => Some(Type::void()),

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

            Self::System_Array_1 => Some(Type::pointer()),
            Self::System_String => Some(Type::pointer()),
            Self::System_Exception => Some(Type::pointer()),
        }
    }
}

pub fn load_stdlib(manager: &AssemblyManager) {
    manager.add_assembly(Assembly::new_for_adding("!".to_owned(), true, |assembly| {
        let assembly = unsafe { assembly.as_ref() };
        vec![
            System_Object(assembly),
            System_ValueType(assembly),
            System_Void(assembly),
        ]
    }));

    let assembly = manager.get_assembly_by_name("!").unwrap().unwrap();
    macro load($i:ident) {{
        let ind = assembly.add_type_handle(($i)(&*assembly));
        debug_assert_eq!(ind, CoreTypeId::$i as u32);
    }}
    load!(System_UInt8);
    load!(System_UInt16);
    load!(System_UInt32);
    load!(System_UInt64);
    load!(System_USize);

    load!(System_Int8);
    load!(System_Int16);
    load!(System_Int32);
    load!(System_Int64);
    load!(System_ISize);

    load!(System_Char);

    load!(System_Array_1);
    load!(System_String);
    load!(System_Exception);
}
