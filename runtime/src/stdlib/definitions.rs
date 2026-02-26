#![allow(unused)]
use std::ptr::NonNull;

use global::attrs::CallConvention;

use proc_macros::{define_core_class, define_core_struct};

#[cfg(feature = "__when_impl")]
use either::Either;

#[cfg(feature = "__when_impl")]
use stdlib_header::CoreTypeId;
#[cfg(any(feature = "__when_impl"))]
use stdlib_header::CoreTypeRef;

#[cfg(feature = "__when_impl")]
use super::{CoreTypeIdConstExt, System, get_core_class, get_core_struct};

#[cfg(feature = "__when_impl")]
use crate::type_system::{
    assembly_manager::AssemblyRef,
    method::{Method, Parameter},
    method_table::MethodTable,
    r#struct::Struct,
    type_handle::{MaybeUnloadedTypeHandle, TypeHandle},
    type_ref::TypeRef,
};

#[cfg(feature = "__when_not_impl")]
#[allow(unused)]
use crate::CoreTypeId;

#[cfg(feature = "__when_serialize")]
use stdlib_header::{CoreTypeId, CoreTypeRef};

#[cfg(feature = "__when_serialize")]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreTypeInfo {
    #[serde(with = "crate::core_type_id_serde")]
    pub id: CoreTypeId,
    pub kind: crate::CoreTypeKind,
    pub attr: global::attrs::TypeAttr,
    pub name: String,
    pub generic_count: Option<crate::GenericCount>,
    #[serde(with = "crate::option_core_type_ref_serde")]
    pub parent: Option<CoreTypeRef>,
    pub methods: Vec<MethodInfo>,
    pub static_methods: Vec<MethodInfo>,
    pub fields: Vec<FieldInfo>,
}

#[cfg(feature = "__when_serialize")]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodInfo {
    pub id: u32,
    pub name: String,
    pub generic_count: Option<crate::GenericCount>,
    pub attr: InfoMethodAttr,
    pub args: Vec<MethodArg>,
    #[serde(with = "crate::core_type_ref_serde")]
    pub return_type: CoreTypeRef,
}

#[cfg(feature = "__when_serialize")]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InfoMethodAttr {
    pub vis: global::attrs::Visibility,
    pub impl_flags: enumflags2::BitFlags<global::attrs::MethodImplementationFlags>,
    pub overrides: Option<u32>,
    #[serde(with = "crate::vec_core_type_ref_serde")]
    pub local_variable_types: Vec<CoreTypeRef>,
}

#[cfg(feature = "__when_serialize")]
impl From<global::attrs::MethodAttr<CoreTypeRef>> for InfoMethodAttr {
    #[inline(always)]
    fn from(
        global::attrs::MethodAttr {
            vis,
            impl_flags,
            overrides,
            local_variable_types,
        }: global::attrs::MethodAttr<CoreTypeRef>,
    ) -> Self {
        Self {
            vis,
            impl_flags,
            overrides,
            local_variable_types,
        }
    }
}

#[cfg(feature = "__when_serialize")]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodArg {
    pub attr: global::attrs::ParameterAttr,
    #[serde(with = "crate::core_type_ref_serde")]
    pub ty: CoreTypeRef,
}

#[cfg(feature = "__when_serialize")]
impl From<(global::attrs::ParameterAttr, CoreTypeRef)> for MethodArg {
    fn from((attr, ty): (global::attrs::ParameterAttr, CoreTypeRef)) -> Self {
        Self { attr, ty }
    }
}

#[cfg(feature = "__when_serialize")]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldInfo {
    pub id: u32,
    pub name: String,
    pub attr: global::attrs::FieldAttr,
    #[serde(with = "crate::core_type_ref_serde")]
    pub ty: CoreTypeRef,
}

define_core_class! {
    #[Public {}] assembly
    System_Object "System::Object" =>
    #fields:

    #methods:
    [
        #[Public {}] Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
    ] [] with
    |mt| vec![
        Box::new(Method::native(
            Some(mt),
            TMethodId::Destructor.get_name().to_owned(),
            TMethodId::Destructor.get_attr(),
            TMethodId::Destructor.get_parameters(),
            TMethodId::Destructor.get_return_type(),
            CallConvention::PlatformDefault,
            None,
            System::Object::Destructor as _,
        )),
        Box::new(Method::native(
            Some(mt),
            TMethodId::ToString.get_name().to_owned(),
            TMethodId::ToString.get_attr(),
            TMethodId::ToString.get_parameters(),
            TMethodId::ToString.get_return_type(),
            CallConvention::PlatformDefault,
            None,
            System::Object::ToString as _,
        )),

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_struct! {
    #[Public {}] assembly
    System_ValueType "System::ValueType" =>
    #fields:

    #methods:
    [] [] with
    |mt| vec![
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_struct! {
    #[Public {}] assembly
    System_Void "System::Void" =>
    #fields:

    #methods:
    [] [] with
    |mt| vec![
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_struct! {
    #[Public {}] assembly
    System_Nullable_1 1 "System::Nullable`1" =>
    #fields:
    #[Private {}] Inner "_Inner" => stdlib_header::CoreTypeRef::Generic(0);

    #methods:
    [] [
        #[Public {Static}] Initialize (
            #[{ByRef}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Nullable_1,
                vec![CoreTypeRef::Generic(0)],
            )
            #[{}] CoreTypeRef::Generic(0)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] with
    |mt| vec![
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
        Box::new(
            Method::native(
                Some(mt),
                "Initialize".to_owned(),
                TStaticMethodId::Initialize.get_attr(),
                TStaticMethodId::Initialize.get_parameters(),
                CoreTypeId::System_Void.static_type_ref().into(),
                CallConvention::PlatformDefault,
                None,
                System::Nullable::Initialize as _,
            )
        ),
    ]
}

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_Boolean "System::Boolean" =>
    [None]
    #fields:
    #methods:
    [] [] with |mt| vec![
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

mod integer {
    #[cfg(feature = "__when_not_impl")]
    use crate::CoreTypeId;

    #[cfg(feature = "__when_impl")]
    use std::ptr::NonNull;

    #[cfg(feature = "__when_impl")]
    use stdlib_header::{CoreTypeId, CoreTypeRef};

    #[cfg(feature = "__when_impl")]
    use crate::stdlib::CoreTypeIdConstExt;
    #[cfg(feature = "__when_impl")]
    use crate::type_system::{
        method::Method, method_table::MethodTable, r#struct::Struct,
        type_handle::MaybeUnloadedTypeHandle,
    };

    #[cfg(feature = "__when_impl")]
    use super::System;

    #[cfg(feature = "__when_serialize")]
    use stdlib_header::{CoreTypeId, CoreTypeRef};

    #[cfg(feature = "__when_serialize")]
    use super::{CoreTypeInfo, FieldInfo, MethodArg, MethodInfo};

    #[cfg(feature = "__when_impl")]
    fn get_int_initializer<T: 'static + std::fmt::Display>(
        id: CoreTypeId,
    ) -> impl Fn(NonNull<MethodTable<Struct>>) -> Vec<Box<Method<Struct>>> {
        move |mt| {
            use crate::type_system::method::{Method, Parameter};

            vec![
                // Statics
                Box::new(Method::default_sctor(
                    Some(mt),
                    global::attr!(
                        method Public {Static}
                    ),
                )),
                Box::new(Method::native(
                    Some(mt),
                    "ToString".to_owned(),
                    global::attr!(
                        method Public {Static}
                    ),
                    vec![Parameter::new(
                        MaybeUnloadedTypeHandle::Unloaded(id.static_type_ref()),
                        global::attr!(parameter { ByRef }),
                    )],
                    MaybeUnloadedTypeHandle::Unloaded(CoreTypeId::System_String.static_type_ref()),
                    global::attrs::CallConvention::PlatformDefault,
                    None,
                    System::_Integers::ToString::<T> as _,
                )),
            ]
        }
    }

    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt8 "System::UInt8" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
            ) -> stdlib_header::CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<u8>(CoreTypeId::System_UInt8)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt16 "System::UInt16" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_UInt16)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<u16>(CoreTypeId::System_UInt16)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt32 "System::UInt32" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_UInt32)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<u32>(CoreTypeId::System_UInt32)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt64 "System::UInt64" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_UInt64)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<u64>(CoreTypeId::System_UInt64)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_USize "System::USize" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_USize)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<usize>(CoreTypeId::System_USize)
    }

    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int8 "System::Int8" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_Int8)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<i8>(CoreTypeId::System_Int8)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int16 "System::Int16" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_Int16)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<i16>(CoreTypeId::System_Int16)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int32 "System::Int32" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}] ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_Int32)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<i32>(CoreTypeId::System_Int32)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int64 "System::Int64" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}]ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_Int64)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<i64>(CoreTypeId::System_Int64)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_ISize "System::ISize" =>
        [None]
        #fields:
        #methods:
        [] [
            #[Public {Static}]ToString (
                #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_ISize)
            ) -> CoreTypeRef::Core(CoreTypeId::System_String);
        ] with get_int_initializer::<isize>(CoreTypeId::System_ISize)
    }
}

pub use integer::*;

define_core_struct! {
    #[Public {}] assembly
    System_Char "System::Char" =>
    #fields:

    #methods:
    [] [] with
    |mt| vec![
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_struct! {
    #[Public {}] assembly
    System_Pointer "System::Pointer" =>
    #fields:
    #[Public {Static}]
    Null "Null" => CoreTypeId::System_Pointer.into();

    #methods:
    [] [] with
    |mt| vec![
        // Statics
        Box::new(
            Method::create_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
                System::Pointer::StaticConstructor,
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_NonPurusCallConfiguration "System::NonPurusCallConfiguration"
    Some(CoreTypeId::System_Object.into()) =>
    #fields:
    #[Public {}]
    CallConvention "CallConvention" => CoreTypeRef::Core(CoreTypeId::System_UInt8);
    #[Public {}]
    ReturnType "ReturnType" => CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
    #[Public {}]
    Encoding "Encoding" => CoreTypeRef::Core(CoreTypeId::System_UInt8);
    #[Public {}]
    ObjectStrategy "ObjectStrategy" => CoreTypeRef::Core(CoreTypeId::System_UInt8);
    #[Public {}]
    ByRefArguments "ByRefArguments" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Array_1,
        vec![
            CoreTypeRef::Core(CoreTypeId::System_USize),
        ],
    );
    #[Public {}]
    Arguments "Arguments" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Array_1,
        vec![
            CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType),
        ],
    );

    #methods of System_Object_MethodId:
    [
        #[Public {}] Constructor ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
            #[{}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![CoreTypeId::System_USize.into()],
            )
            #[{}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![CoreTypeId::System_NonPurusCallType.into()],
            )
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Constructor.get_name().to_owned(),
                TMethodId::Constructor.get_attr(),
                TMethodId::Constructor.get_parameters(),
                TMethodId::Constructor.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::NonPurusCallConfiguration::Constructor as _,
            )
        ),
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_NonPurusCallType "System::NonPurusCallType" Some(CoreTypeId::System_Object.into()) =>
    #fields of System_Object_FieldId:
    #[Public {}]
    Discriminant "Discriminant" => CoreTypeId::System_UInt8.into();
    #[Public {}]
    Types "Types" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Nullable_1,
        vec![
            CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![
                    CoreTypeId::System_NonPurusCallType.into(),
                ],
            )
        ],
    );

    #methods of System_Object_MethodId:
    [] [
        #[Public {Static}] CreateVoid () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU8 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI8 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU16 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI16 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU32 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI32 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU64 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI64 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreatePointer () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateString () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateObject () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateStructure (
            #[{}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![
                    CoreTypeId::System_NonPurusCallType.into(),
                ],
            )
        ) -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
    ] with
    |mt| {
        macro def($x:ident) {
            Box::new(Method::native(
                Some(mt),
                concat!("Create", stringify!($x)).to_owned(),
                TStaticMethodId::${concat(Create, $x)}.get_attr(),
                TStaticMethodId::${concat(Create, $x)}.get_parameters(),
                TStaticMethodId::${concat(Create, $x)}.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::NonPurusCallType::${concat(Create, $x)} as _,
            ))
        }
        vec![
            // Statics
            Box::new(
                Method::default_sctor(
                    Some(mt),
                    TStaticMethodId::StaticConstructor.get_attr(),
                ),
            ),
            def!(Void),

            def!(U8),
            def!(I8),

            def!(U16),
            def!(I16),

            def!(U32),
            def!(I32),

            def!(U64),
            def!(I64),

            def!(Pointer),

            def!(String),
            def!(Object),

            Box::new(Method::native(
                Some(mt),
                TStaticMethodId::CreateStructure.get_name().to_owned(),
                TStaticMethodId::CreateStructure.get_attr(),
                TStaticMethodId::CreateStructure.get_parameters(),
                TStaticMethodId::CreateStructure.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::NonPurusCallType::CreateStructure as _,
            )),
        ]
    }
}

define_core_class! {
    #[Public {}] assembly
    System_DynamicLibrary "System::DynamicLibrary" Some(CoreTypeId::System_Object.into()) =>
    #fields of System_Object_FieldId:
    #[Private {}] Handle "_handle" => CoreTypeId::System_Pointer.into();

    #methods of System_Object_MethodId:
    [
        #[override Some(System_Object_MethodId::Destructor as _) Public {}]
        Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] Constructor_String ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] GetSymbol "GetSymbol" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
    ] [] with |mt| {
        vec![
            Box::new(
                Method::native(
                    Some(mt),
                    TMethodId::Destructor.get_name().to_owned(),
                    TMethodId::Destructor.get_attr(),
                    TMethodId::Destructor.get_parameters(),
                    TMethodId::Destructor.get_return_type(),
                    CallConvention::PlatformDefault,
                    None,
                    System::DynamicLibrary::Destructor as _,
                ),
            ),
            Box::new(
                Method::native(
                    Some(mt),
                    TMethodId::Constructor_String.get_name().to_owned(),
                    TMethodId::Constructor_String.get_attr(),
                    TMethodId::Constructor_String.get_parameters(),
                    TMethodId::Constructor_String.get_return_type(),
                    CallConvention::PlatformDefault,
                    None,
                    System::DynamicLibrary::Constructor_String as _,
                )
            ),
            Box::new(
                Method::native(
                    Some(mt),
                    TMethodId::GetSymbol.get_name().to_owned(),
                    TMethodId::GetSymbol.get_attr(),
                    TMethodId::GetSymbol.get_parameters(),
                    TMethodId::GetSymbol.get_return_type(),
                    CallConvention::PlatformDefault,
                    None,
                    System::DynamicLibrary::GetSymbol as _,
                )
            ),
            // Statics
            Box::new(
                Method::default_sctor(
                    Some(mt),
                    TStaticMethodId::StaticConstructor.get_attr(),
                ),
            ),
        ]
    }
}

define_core_class! {
    #[Public {}] assembly
    System_Array_1 1 "System::Array`1" Some(CoreTypeId::System_Object.into()) =>
    #fields:

    #methods of System_Object_MethodId:
    [
        #[override Some(System_Object_MethodId::Destructor as _) Public {}]
        Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[override Some(System_Object_MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
        #[Private {}] GetPointerOfIndex (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
        #[
            Public {}
            CoreTypeRef::Core(CoreTypeId::System_Object),
            CoreTypeRef::Core(CoreTypeId::System_USize), // arg 0
            CoreTypeRef::Core(CoreTypeId::System_Pointer),
            CoreTypeRef::Core(CoreTypeId::System_USize), // Size of T
            CoreTypeRef::Generic(0),
        ] get_Index (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
        ) -> CoreTypeRef::Generic(0);
        #[
            Public {}
            CoreTypeRef::Core(CoreTypeId::System_Object), // this
            CoreTypeRef::Core(CoreTypeId::System_USize), // arg 0
            CoreTypeRef::Generic(0), // arg 1
            CoreTypeRef::Core(CoreTypeId::System_USize), // size of T
            CoreTypeRef::Core(CoreTypeId::System_Pointer), // pointer of result
        ] set_Index (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
            #[{}] CoreTypeRef::Generic(0)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Destructor.get_name().to_owned(),
                TMethodId::Destructor.get_attr(),
                TMethodId::Destructor.get_parameters(),
                TMethodId::Destructor.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::Array_1::Destructor as _,
            ),
        ),
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::ToString.get_name().to_owned(),
                TMethodId::ToString.get_attr(),
                TMethodId::ToString.get_parameters(),
                TMethodId::ToString.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::Array_1::ToString as _,
            ),
        ),
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::GetPointerOfIndex.get_name().to_owned(),
                TMethodId::GetPointerOfIndex.get_attr(),
                TMethodId::GetPointerOfIndex.get_parameters(),
                TMethodId::GetPointerOfIndex.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::Array_1::GetPointerOfIndex as _,
            ),
        ),
        Box::new(
            Method::new(
                mt,
                TMethodId::get_Index.get_name().to_owned(),
                TMethodId::get_Index.get_attr(),
                TMethodId::get_Index.get_parameters(),
                TMethodId::get_Index.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                {
                    use global::instruction::Instruction;
                    vec![
                        Instruction::LoadThis {
                            register_addr: 0,
                        },
                        Instruction::LoadArg {
                            register_addr: 1,
                            arg: 0,
                        },
                        Instruction::InstanceCall {
                            val: 0,
                            method: System_Array_1_MethodId::GetPointerOfIndex.into(),
                            args: vec![1],
                            ret_at: 2,
                        },
                        Instruction::LoadTypeValueSize {
                            register_addr: 3,
                            ty: TypeHandle::Generic(0).into(),
                        },
                        Instruction::ReadPointerTo {
                            ptr: 2,
                            size: 3,
                            destination: 4,
                        },
                        Instruction::ReturnVal { register_addr: 4 },
                    ]
                },
            )
        ),
        Box::new(
            Method::new(
                mt,
                TMethodId::set_Index.get_name().to_owned(),
                TMethodId::set_Index.get_attr(),
                TMethodId::set_Index.get_parameters(),
                TMethodId::set_Index.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                {
                    use global::instruction::Instruction;
                    vec![
                        Instruction::LoadThis {
                            register_addr: 0,
                        },
                        Instruction::LoadArg {
                            register_addr: 1,
                            arg: 0,
                        },
                        Instruction::LoadArg {
                            register_addr: 2,
                            arg: 1,
                        },

                        Instruction::LoadTypeValueSize {
                            register_addr: 3,
                            ty: TypeHandle::Generic(0).into(),
                        },
                        Instruction::InstanceCall {
                            val: 0,
                            method: System_Array_1_MethodId::GetPointerOfIndex.into(),
                            args: vec![1],
                            ret_at: 4,
                        },
                        Instruction::WritePointer { source: 2, size: 3, ptr: 4 }
                    ]
                },
            )
        ),
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_String "System::String" Some(CoreTypeId::System_Object.into()) =>
    #fields:

    #methods of System_Object_MethodId:
    [
        #[override Some(System_Object_MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
        #[Public {}] get_Length () -> CoreTypeRef::Core(CoreTypeId::System_USize);
        #[Public {}] get_U32Length () -> CoreTypeRef::Core(CoreTypeId::System_UInt32);
    ] [] with
    |mt| vec![
        Box::new(Method::native(
            Some(mt),
            TMethodId::ToString.get_name().to_owned(),
            TMethodId::ToString.get_attr(),
            TMethodId::ToString.get_parameters(),
            TMethodId::ToString.get_return_type(),
            CallConvention::PlatformDefault,
            None,
            System::String::ToString as _,
        )),
        Box::new(Method::native(
            Some(mt),
            TMethodId::get_Length.get_name().to_owned(),
            TMethodId::get_Length.get_attr(),
            TMethodId::get_Length.get_parameters(),
            TMethodId::get_Length.get_return_type(),
            CallConvention::PlatformDefault,
            None,
            System::String::get_Length as _,
        )),
        Box::new(Method::native(
            Some(mt),
            TMethodId::get_U32Length.get_name().to_owned(),
            TMethodId::get_U32Length.get_attr(),
            TMethodId::get_U32Length.get_parameters(),
            TMethodId::get_U32Length.get_return_type(),
            CallConvention::PlatformDefault,
            None,
            System::String::get_U32Length as _,
        )),

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_LargeString "System::LargeString" Some(CoreTypeId::System_Object.into()) =>
    #fields:

    #methods of System_Object_MethodId:
    [
        #[override Some(System_Object_MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
    ] [] with
    |mt| vec![
        Box::new(Method::native(
            Some(mt),
            TMethodId::ToString.get_name().to_owned(),
            TMethodId::ToString.get_attr(),
            TMethodId::ToString.get_parameters(),
            TMethodId::ToString.get_return_type(),
            CallConvention::PlatformDefault,
            None,
            System::LargeString::ToString as _,
        )),

        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_Environment "System::Environment" Some(CoreTypeId::System_Object.into()) =>
    #fields of System_Object_FieldId:
    #[Public {}] NewLine "NewLine" => CoreTypeId::System_String.into();

    #methods of System_Object_MethodId:
    [] [] with
    |mt| vec![
        // Statics
        Box::new(Method::create_sctor(
            Some(mt),
            TStaticMethodId::StaticConstructor.get_attr(),
            System::Environment::StaticConstructor,
        )),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_Exception "System::Exception" Some(CoreTypeId::System_Object.into()) =>
    #fields of System_Object_FieldId:
    #[Public {}] Message "_message" => CoreTypeId::System_String.into();
    #[Public {}] Inner "_innerException" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Nullable_1,
        vec![
            CoreTypeId::System_Exception.into(),
        ],
    );
    #[Public {}] StackTrace "_stackTrace" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Array_1,
        vec![
            CoreTypeId::System_String.into(),
        ],
    );

    #methods of System_Object_MethodId:
    [
        #[override Some(System_Object_MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
        #[Public {HideWhenCapturing}] Constructor_String ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::ToString.get_name().to_owned(),
                TMethodId::ToString.get_attr(),
                TMethodId::ToString.get_parameters(),
                TMethodId::ToString.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::Exception::ToString as _,
            ),
        ),
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Constructor_String.get_name().to_owned(),
                TMethodId::Constructor_String.get_attr(),
                TMethodId::Constructor_String.get_parameters(),
                TMethodId::Constructor_String.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::Exception::Construct_String as _,
            )
        ),
        // Statics
        Box::new(
            Method::default_sctor(
                Some(mt),
                TStaticMethodId::StaticConstructor.get_attr(),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_InvalidEnumException "System::InvalidEnumException" Some(CoreTypeId::System_Exception.into()) =>
    #fields:

    #methods of System_Exception_MethodId:
    [
        #[Public {}] Constructor_String_String ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Constructor_String_String.get_name().to_owned(),
                TMethodId::Constructor_String_String.get_attr(),
                TMethodId::Constructor_String_String.get_parameters(),
                TMethodId::Constructor_String_String.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::InvalidEnumException::Constructor_String_String as _,
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

define_core_class! {
    #[Public {}] assembly
    System_Win32Exception "System::Win32Exception" Some(CoreTypeId::System_Exception.into()) =>
    #fields of System_Exception_FieldId:
    #[Public {}] Code "_Code" => CoreTypeId::System_Int32.into();

    #methods of System_Exception_MethodId:
    [
        #[Public {}] Constructor ".ctor" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] Constructor_I32 ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Int32)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Constructor.get_name().to_owned(),
                TMethodId::Constructor.get_attr(),
                TMethodId::Constructor.get_parameters(),
                TMethodId::Constructor.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::Win32Exception::Constructor as _,
            )
        ),
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Constructor_I32.get_name().to_owned(),
                TMethodId::Constructor_I32.get_attr(),
                TMethodId::Constructor_I32.get_parameters(),
                TMethodId::Constructor_I32.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::Win32Exception::Constructor_I32 as _,
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

define_core_class! {
    #[Public {}] assembly
    System_ErrnoException "System::ErrnoException" Some(CoreTypeId::System_Exception.into()) =>
    #fields of System_Exception_FieldId:
    #[Public {}] Code "_Code" => CoreTypeId::System_Int32.into();

    #methods of System_Exception_MethodId:
    [
        #[Public {}] Constructor ".ctor" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] Constructor_I32 ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Int32)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Constructor.get_name().to_owned(),
                TMethodId::Constructor.get_attr(),
                TMethodId::Constructor.get_parameters(),
                TMethodId::Constructor.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::ErrnoException::Constructor as _,
            )
        ),
        Box::new(
            Method::native(
                Some(mt),
                TMethodId::Constructor_I32.get_name().to_owned(),
                TMethodId::Constructor_I32.get_attr(),
                TMethodId::Constructor_I32.get_parameters(),
                TMethodId::Constructor_I32.get_return_type(),
                CallConvention::PlatformDefault,
                None,
                System::ErrnoException::Constructor_I32 as _,
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

define_core_class! {
    #[Public {}] assembly
    System_DlErrorException "System::DlErrorException" Some(CoreTypeId::System_Exception.into()) =>
    #fields:

    #methods of System_Exception_MethodId:
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
