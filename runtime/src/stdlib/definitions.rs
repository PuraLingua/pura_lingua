#![allow(unused)]
use std::ptr::NonNull;

use global::attrs::CallConvention;

use proc_macros::{define_core_class, define_core_struct, when_impl, when_not_impl};

when_impl! {
    use either::Either;

    use stdlib_header::CoreTypeId;

    use crate::type_system::{
        assembly_manager::AssemblyRef,
        method::{Method, Parameter},
        type_handle::{MaybeUnloadedTypeHandle, TypeHandle},
        type_ref::TypeRef,
        method_table::MethodTable,
        r#struct::Struct,
    };
    use super::{System, CoreTypeIdConstExt, get_core_class, get_core_struct};
}

when_not_impl! {
    #[allow(unused)]
    use crate::CoreTypeId;
}

define_core_class! {
    #[Public {}] assembly
    System_Object "System::Object" =>
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
            "ToString".to_owned(),
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
    System_ValueType "System::ValueType" =>
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
    System_Void "System::Void" =>
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
    System_Nullable_1 "System::Nullable`1" =>
    #fields:
    #[Private {}] Inner "_Inner" => MaybeUnloadedTypeHandle::Loaded(TypeHandle::Generic(0));

    #methods:
    [] [Initialize] with
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
        Box::new(
            Method::native(
                Some(mt),
                "Initialize".to_owned(),
                global::attr!(
                    method Public {Static}
                ),
                vec![
                    Parameter::new(
                        CoreTypeId::System_Nullable_1.static_type_ref().into(),
                        global::attr!(
                            parameter {ByRef}
                        )
                    ),
                    Parameter::new(
                        MaybeUnloadedTypeHandle::Loaded(TypeHandle::Generic(0)),
                        global::attr!(
                            parameter {}
                        )
                    ),
                ],
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
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

mod integer {
    use proc_macros::{when_impl, when_not_impl};

    when_not_impl!(
        use crate::CoreTypeId;
    );

    when_impl! {
        use std::ptr::NonNull;

        use stdlib_header::CoreTypeId;

        use crate::type_system::{
            method_table::MethodTable,
            r#struct::Struct,
            method::Method,
            type_handle::MaybeUnloadedTypeHandle,
        };
        use crate::stdlib::CoreTypeIdConstExt;

        use super::System;

        fn get_int_initializer<T: 'static + std::fmt::Display>(id: CoreTypeId)
            -> impl Fn(NonNull<MethodTable<Struct>>) -> Vec<Box<Method<Struct>>> {
            move |mt| {
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
                        "ToString".to_owned(),
                        global::attr!(
                            method Public {Static}
                        ),
                        vec![
                            Parameter::new(
                                MaybeUnloadedTypeHandle::Unloaded(id.static_type_ref()),
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
                        System::_Integers::ToString::<T> as _,
                    )),
                ]
            }
        }
    }

    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt8 "System::UInt8" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<u8>(CoreTypeId::System_UInt8)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt16 "System::UInt16" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<u16>(CoreTypeId::System_UInt16)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt32 "System::UInt32" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<u32>(CoreTypeId::System_UInt32)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_UInt64 "System::UInt64" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<u64>(CoreTypeId::System_UInt64)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_USize "System::USize" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<usize>(CoreTypeId::System_USize)
    }

    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int8 "System::Int8" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<i8>(CoreTypeId::System_Int8)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int16 "System::Int16" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<i16>(CoreTypeId::System_Int16)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int32 "System::Int32" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<i32>(CoreTypeId::System_Int32)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_Int64 "System::Int64" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<i64>(CoreTypeId::System_Int64)
    }
    proc_macros::define_core_struct! {
        #[Public {}] assembly
        System_ISize "System::ISize" =>
        [None]
        #fields:
        #methods:
        [] [ToString] with get_int_initializer::<isize>(CoreTypeId::System_ISize)
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
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

define_core_struct! {
    #[Public {}] assembly
    System_Pointer "System::Pointer" =>
    #fields:
    #[Public {Static}]
    Null "Null" => CoreTypeId::System_Pointer.static_type_ref().into();

    #methods:
    [] [] with
    |mt| vec![
        // Statics
        Box::new(
            Method::create_sctor(
                Some(mt),
                global::attr!(
                    method Public {Static}
                ),
                System::Pointer::StaticConstructor,
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_NonPurusCallConfiguration "System::NonPurusCallConfiguration"
    Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields:
    #[Public {}]
    CallConvention "CallConvention" => CoreTypeId::System_UInt8.static_type_ref().into();
    #[Public {}]
    ReturnType "ReturnType" => CoreTypeId::System_NonPurusCallType.static_type_ref().into();
    #[Public {}]
    Encoding "Encoding" => CoreTypeId::System_UInt8.static_type_ref().into();
    #[Public {}]
    ObjectStrategy "ObjectStrategy" => CoreTypeId::System_UInt8.static_type_ref().into();
    #[Public {}]
    ByRefArguments "ByRefArguments" => MaybeUnloadedTypeHandle::Unloaded(
        TypeRef::Specific {
            assembly_and_index: Either::Right(Box::new(CoreTypeId::System_Array_1.static_type_ref().into())),
            types: vec![
                CoreTypeId::System_USize.static_type_ref().into(),
            ],
        },
    );
    #[Public {}]
    Arguments "Arguments" => MaybeUnloadedTypeHandle::Unloaded(
        TypeRef::Specific {
            assembly_and_index: Either::Right(Box::new(CoreTypeId::System_Array_1.static_type_ref().into())),
            types: vec![
                CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
            ],
        },
    );

    #methods of System_Object_MethodId:
    [Constructor] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                ".ctor".to_owned(),
                global::attr!(
                    method Public {}
                ),
                vec![
                    Parameter::new(
                        CoreTypeId::System_UInt8.static_type_ref().into(),
                        global::attr!(parameter {}),
                    ),
                    Parameter::new(
                        CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                        global::attr!(parameter {}),
                    ),
                    Parameter::new(
                        CoreTypeId::System_UInt8.static_type_ref().into(),
                        global::attr!(parameter {}),
                    ),
                    Parameter::new(
                        CoreTypeId::System_UInt8.static_type_ref().into(),
                        global::attr!(parameter {}),
                    ),
                    Parameter::new(
                        MaybeUnloadedTypeHandle::Unloaded(
                            TypeRef::Specific {
                                assembly_and_index: Either::Right(Box::new(CoreTypeId::System_Array_1.static_type_ref().into())),
                                types: vec![
                                    CoreTypeId::System_USize.static_type_ref().into(),
                                ],
                            },
                        ),
                        global::attr!(parameter {}),
                    ),
                    Parameter::new(
                        MaybeUnloadedTypeHandle::Unloaded(
                            TypeRef::Specific {
                                assembly_and_index: Either::Right(Box::new(
                                    CoreTypeId::System_Array_1.static_type_ref().into(),
                                )),
                                types: vec![
                                    CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                                ],
                            },
                        ),
                        global::attr!(parameter {}),
                    ),
                ],
                CoreTypeId::System_Void.static_type_ref().into(),
                CallConvention::PlatformDefault,
                None,
                System::NonPurusCallConfiguration::Constructor as _,
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
    System_NonPurusCallType "System::NonPurusCallType" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields of System_Object_FieldId:
    #[Public {}]
    Discriminant "Discriminant" => CoreTypeId::System_UInt8.static_type_ref().into();
    #[Public {}]
    Types "Types" => MaybeUnloadedTypeHandle::Unloaded(
        TypeRef::Specific {
            assembly_and_index: Either::Right(Box::new(
                CoreTypeId::System_Nullable_1.static_type_ref().into()
            )),
            types: vec![
                MaybeUnloadedTypeHandle::Unloaded(
                    TypeRef::Specific {
                        assembly_and_index: Either::Right(
                            Box::new(
                                CoreTypeId::System_Array_1.static_type_ref().into(),
                            ),
                        ),
                        types: vec![
                            CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                        ]
                    },
                )
            ],
        },
    );

    #methods of System_Object_MethodId:
    [] [
        CreateVoid

        CreateU8
        CreateI8

        CreateU16
        CreateI16

        CreateU32
        CreateI32

        CreateU64
        CreateI64

        CreatePointer

        CreateString
        CreateObject

        CreateStructure
    ] with
    |mt| {
        macro def($x:ident) {
            Box::new(Method::native(
                Some(mt),
                concat!("Create", stringify!($x)).to_owned(),
                global::attr!(
                    method Public {Static}
                ),
                vec![],
                CoreTypeId::System_NonPurusCallType
                    .static_type_ref()
                    .into(),
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
                    global::attr!(
                        method Public {Static}
                    ),
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
                "CreateStructure".to_owned(),
                global::attr!(
                    method Public {Static}
                ),
                vec![
                    Parameter::new(
                        MaybeUnloadedTypeHandle::Unloaded(
                            TypeRef::Specific {
                                assembly_and_index: Either::Right(
                                    Box::new(
                                        CoreTypeId::System_Array_1
                                            .static_type_ref()
                                            .into(),
                                        ),
                                    ),
                                types: vec![
                                    CoreTypeId::System_NonPurusCallType
                                        .static_type_ref()
                                        .into()
                                ],
                            },
                        ),
                        global::attr!(
                            parameter {}
                        )
                    ),
                ],
                CoreTypeId::System_NonPurusCallType
                    .static_type_ref()
                    .into(),
                CallConvention::PlatformDefault,
                None,
                System::NonPurusCallType::CreateStructure as _,
            )),
        ]
    }
}

define_core_class! {
    #[Public {}] assembly
    System_DynamicLibrary "System::DynamicLibrary" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields of System_Object_FieldId:
    #[Private {}] Handle "_handle" => CoreTypeId::System_Pointer.static_type_ref().into();

    #methods of System_Object_MethodId:
    [
        override Destructor
        Constructor_String
        GetSymbol
    ] [] with |mt| {
        vec![
            Box::new(
                Method::native(
                    Some(mt),
                    "~".to_owned(),
                    global::attr!(
                        method override Some(System_Object_MethodId::Destructor as _) Public {}
                    ),
                    vec![],
                    get_core_struct(CoreTypeId::System_Void, assembly).into(),
                    CallConvention::PlatformDefault,
                    None,
                    System::DynamicLibrary::Destructor as _,
                ),
            ),
            Box::new(
                Method::native(
                    Some(mt),
                    ".ctor".to_owned(),
                    global::attr!(
                        method Public {}
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
                    System::DynamicLibrary::Constructor_String as _,
                )
            ),
            Box::new(
                Method::native(
                    Some(mt),
                    "GetSymbol".to_owned(),
                    global::attr!(
                        method Public {}
                    ),
                    vec![
                        Parameter::new(
                            CoreTypeId::System_String.static_type_ref().into(),
                            global::attr!(
                                parameter {}
                            ),
                        ),
                    ],
                    CoreTypeId::System_Pointer.static_type_ref().into(),
                    CallConvention::PlatformDefault,
                    None,
                    System::DynamicLibrary::GetSymbol as _,
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
}

define_core_class! {
    #[Public {}] assembly
    System_Array_1 "System::Array`1" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields:

    #methods of System_Object_MethodId:
    [
        override Destructor
        override ToString
        GetPointerOfIndex
        get_Index
        set_Index
    ] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                "~".to_owned(),
                global::attr!(
                    method override Some(System_Object_MethodId::Destructor as _) Public {}
                ),
                vec![],
                get_core_struct(CoreTypeId::System_Void, assembly).into(),
                CallConvention::PlatformDefault,
                None,
                System::Array_1::Destructor as _,
            ),
        ),
        Box::new(
            Method::native(
                Some(mt),
                "ToString".to_owned(),
                global::attr!(
                    method override Some(System_Object_MethodId::ToString as _) Public {}
                ),
                vec![],
                CoreTypeId::System_String.static_type_ref().into(),
                CallConvention::PlatformDefault,
                None,
                System::Array_1::ToString as _,
            ),
        ),
        Box::new(
            Method::native(
                Some(mt),
                "GetPointerOfIndex".to_owned(),
                global::attr!(
                    method Private {}
                ),
                vec![
                    Parameter::with_core_type(
                        CoreTypeId::System_USize,
                    ),
                ],
                CoreTypeId::System_Pointer.static_type_ref().into(),
                CallConvention::PlatformDefault,
                None,
                System::Array_1::GetPointerOfIndex as _,
            ),
        ),
        Box::new(
            Method::new(
                mt,
                "get_Index".to_owned(),
                global::attr!(
                    method Public {}
                    CoreTypeId::System_Object.static_type_ref().into(),
                    CoreTypeId::System_USize.static_type_ref().into(), // arg 0
                    CoreTypeId::System_Pointer.static_type_ref().into(),
                    CoreTypeId::System_USize.static_type_ref().into(), // Size of T
                    TypeHandle::Generic(0).into(),
                ),
                vec![
                    Parameter::with_core_type(
                        CoreTypeId::System_USize,
                    ),
                ],
                TypeHandle::Generic(0).into(),
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
                "set_Index".to_owned(),
                global::attr!(
                    method Public {}
                    CoreTypeId::System_Object.static_type_ref().into(), // this
                    CoreTypeId::System_USize.static_type_ref().into(), // arg 0
                    TypeHandle::Generic(0).into(), // arg 1
                    CoreTypeId::System_USize.static_type_ref().into(), // size of T
                    CoreTypeId::System_Pointer.static_type_ref().into(), // pointer of result
                ),
                vec![
                    Parameter::with_core_type(
                        CoreTypeId::System_USize,
                    ),
                    Parameter::new(
                        TypeHandle::Generic(0).into(),
                        global::attr!(
                            parameter {}
                        )
                    ),
                ],
                CoreTypeId::System_Void.static_type_ref().into(),
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
                global::attr!(
                    method Public {Static}
                ),
            ),
        ),
    ]
}

define_core_class! {
    #[Public {}] assembly
    System_String "System::String" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields:

    #methods of System_Object_MethodId:
    [override ToString get_Length get_U32Length] [] with
    |mt| vec![
        Box::new(Method::native(
            Some(mt),
            "ToString".to_owned(),
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
        Box::new(Method::native(
            Some(mt),
            "get_Length".to_owned(),
            global::attr!(
                method Public {}
            ),
            vec![],
            MaybeUnloadedTypeHandle::Unloaded(
                CoreTypeId::System_USize.static_type_ref(),
            ),
            CallConvention::PlatformDefault,
            None,
            System::String::get_Length as _,
        )),
        Box::new(Method::native(
            Some(mt),
            "get_U32Length".to_owned(),
            global::attr!(
                method Public {}
            ),
            vec![],
            MaybeUnloadedTypeHandle::Unloaded(
                CoreTypeId::System_UInt32.static_type_ref(),
            ),
            CallConvention::PlatformDefault,
            None,
            System::String::get_U32Length as _,
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
    System_LargeString "System::LargeString" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields:

    #methods of System_Object_MethodId:
    [override ToString] [] with
    |mt| vec![
        Box::new(Method::native(
            Some(mt),
            "ToString".to_owned(),
            global::attr!(
                method override Some(System_Object_MethodId::ToString as _) Public {}
            ),
            vec![],
            MaybeUnloadedTypeHandle::Unloaded(
                CoreTypeId::System_String.static_type_ref(),
            ),
            CallConvention::PlatformDefault,
            None,
            System::LargeString::ToString as _,
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
    System_Exception "System::Exception" Some(get_core_class(CoreTypeId::System_Object, assembly)) =>
    #fields of System_Object_FieldId:
    #[Public {}] Message "_message" => CoreTypeId::System_String.static_type_ref().into();
    #[Public {}] Inner "_innerException" => TypeRef::Specific {
        assembly_and_index: Either::Left((AssemblyRef::CORE, CoreTypeId::System_Nullable_1 as _)),
        types: vec![
            CoreTypeId::System_Exception.static_type_ref().into(),
        ]
    }.into();
    #[Public {}] StackTrace "_stackTrace" => TypeRef::Specific {
        assembly_and_index: Either::Left((AssemblyRef::CORE, CoreTypeId::System_Array_1 as _)),
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
                "ToString".to_owned(),
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

define_core_class! {
    #[Public {}] assembly
    System_InvalidEnumException "System::InvalidEnumException" Some(get_core_class(CoreTypeId::System_Exception, assembly)) =>
    #fields:

    #methods of System_Exception_MethodId:
    [Constructor_String_String] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                ".ctor".to_owned(),
                global::attr!(
                    method Public {}
                ),
                vec![
                    Parameter::new(
                        get_core_class(CoreTypeId::System_String, assembly).into(),
                        global::attr!(
                            parameter {}
                        )
                    ),
                    Parameter::new(
                        get_core_class(CoreTypeId::System_String, assembly).into(),
                        global::attr!(
                            parameter {}
                        )
                    ),
                ],
                get_core_struct(CoreTypeId::System_Void, assembly).into(),
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
    System_Win32Exception "System::Win32Exception" Some(get_core_class(CoreTypeId::System_Exception, assembly)) =>
    #fields of System_Exception_FieldId:
    #[Public {}] Code "_Code" => CoreTypeId::System_Int32.static_type_ref().into();

    #methods of System_Exception_MethodId:
    [Constructor Constructor_I32] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                ".ctor".to_owned(),
                global::attr!(
                    method Public {}
                ),
                vec![],
                get_core_struct(CoreTypeId::System_Void, assembly).into(),
                CallConvention::PlatformDefault,
                None,
                System::Win32Exception::Constructor as _,
            )
        ),
        Box::new(
            Method::native(
                Some(mt),
                ".ctor".to_owned(),
                global::attr!(
                    method Public {}
                ),
                vec![
                    Parameter::new(
                        get_core_struct(CoreTypeId::System_Int32, assembly).into(),
                        global::attr!(
                            parameter {}
                        )
                    ),
                ],
                get_core_struct(CoreTypeId::System_Void, assembly).into(),
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
    System_ErrnoException "System::ErrnoException" Some(get_core_class(CoreTypeId::System_Exception, assembly)) =>
    #fields of System_Exception_FieldId:
    #[Public {}] Code "_Code" => CoreTypeId::System_Int32.static_type_ref().into();

    #methods of System_Exception_MethodId:
    [Constructor Constructor_I32] [] with
    |mt| vec![
        Box::new(
            Method::native(
                Some(mt),
                ".ctor".to_owned(),
                global::attr!(
                    method Public {}
                ),
                vec![],
                get_core_struct(CoreTypeId::System_Void, assembly).into(),
                CallConvention::PlatformDefault,
                None,
                System::ErrnoException::Constructor as _,
            )
        ),
        Box::new(
            Method::native(
                Some(mt),
                ".ctor".to_owned(),
                global::attr!(
                    method Public {}
                ),
                vec![
                    Parameter::new(
                        get_core_struct(CoreTypeId::System_Int32, assembly).into(),
                        global::attr!(
                            parameter {}
                        )
                    ),
                ],
                get_core_struct(CoreTypeId::System_Void, assembly).into(),
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
    System_DlErrorException "System::DlErrorException" Some(get_core_class(CoreTypeId::System_Exception, assembly)) =>
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
