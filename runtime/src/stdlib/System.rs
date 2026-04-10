use std::ptr::NonNull;

use global::attrs::{MethodAttr, ParameterAttr};
use stdlib_header::{CoreTypeId, CoreTypeInfo, CoreTypeKind, CoreTypeRef, FieldInfo, MethodInfo};

use crate::type_system::{
    assembly::Assembly,
    class::Class,
    field::Field,
    method::{Method, Parameter},
    method_table::MethodTable,
    r#struct::Struct,
    type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
};

pub mod Array_1;
pub mod Boolean;
pub mod Char;
pub mod DlErrorException;
pub mod DynamicLibrary;
pub mod Environment;
pub mod ErrnoException;
pub mod Exception;
pub mod InvalidEnumException;
pub mod LargeString;
pub mod NonPurusCallConfiguration;
pub mod NonPurusCallType;
pub mod Nullable_1;
pub mod Object;
pub mod Pointer;
pub mod String;
pub mod Tuple;
pub mod ValueType;
pub mod Void;
pub mod Win32Exception;
pub mod _Integers;

pub(crate) macro common_new_method($mt:ident $TMethodId:ident $id:ident $f:path) {
    Box::new(Method::native(
        Some($mt),
        $TMethodId::$id.get_name().to_owned(),
        $crate::stdlib::System::map_method_attr($TMethodId::$id.get_attr()),
        $TMethodId::$id
            .get_parameters()
            .into_iter()
            .map($crate::stdlib::System::map_parameter)
            .collect(),
        $TMethodId::$id.get_return_type().into(),
        ::global::attrs::CallConvention::PlatformDefault,
        None,
        $f as _,
    ))
}

pub(crate) macro default_sctor($mt:ident $TMethodId:ident) {
    Box::new(Method::default_sctor(
        Some($mt),
        $crate::stdlib::System::map_method_attr($TMethodId::StaticConstructor.get_attr()),
    ))
}

fn map_method_attr(val: MethodAttr<CoreTypeRef>) -> MethodAttr<MaybeUnloadedTypeHandle> {
    val.map_types(MaybeUnloadedTypeHandle::from)
}

fn map_parameter((attr, ty): (ParameterAttr, CoreTypeRef)) -> Parameter {
    Parameter::new(ty.into(), attr)
}

fn map_field_info(
    FieldInfo {
        id: _,
        name,
        attr,
        ty,
    }: FieldInfo,
) -> Field {
    Field::new(name, attr, ty.into())
}

macro _define_class(
    fn $load:ident($assembly:ident, $mt:ident, $method_info:ident)
    $id:ident
#methods($TMethodId:ident):
    $(
        $MethodName:ident => $f:expr;
    )*
#static_methods($TStaticMethodId:ident):
    $(
        $StaticMethodName:ident => $static_f:expr;
    )*
) {
    impl From<::stdlib_header::System::$id::MethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::$id::MethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    impl From<::stdlib_header::System::$id::StaticMethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::$id::StaticMethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    pub fn $load(
        $assembly: &$crate::type_system::assembly::Assembly,
    )
    -> $crate::type_system::type_handle::NonGenericTypeHandle {
        type $TMethodId = ::stdlib_header::System::$id::MethodId;
        type $TStaticMethodId = ::stdlib_header::System::$id::StaticMethodId;
        $crate::stdlib::System::define_class(
            ::stdlib_header::CoreTypeId::${concat(System_, $id)},
            |#[allow(unused)] $mt, #[allow(unused)] $method_info| match unsafe { $method_info.get_id::<$TMethodId>() } {
                $(
                    $TMethodId::$MethodName => $f,
                )*
                $TMethodId::__END => unreachable!(),
            },
            |#[allow(unused)] $mt, #[allow(unused)] $method_info| match unsafe { $method_info.get_id::<$TStaticMethodId>() } {
                $(
                    $TStaticMethodId::$StaticMethodName => $static_f,
                )*
                $TStaticMethodId::__END => unreachable!(),
            },
        )($assembly)
    }
}

macro _define_struct(
    fn $load:ident($assembly:ident, $mt:ident, $method_info:ident)
    $id:ident
#methods($TMethodId:ident):
    $(
        $MethodName:ident => $f:expr;
    )*
#static_methods($TStaticMethodId:ident):
    $(
        $StaticMethodName:ident => $static_f:expr;
    )*
) {
    impl From<::stdlib_header::System::$id::MethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::$id::MethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    impl From<::stdlib_header::System::$id::StaticMethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::$id::StaticMethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    pub fn $load(
        $assembly: &$crate::type_system::assembly::Assembly,
    )
    -> $crate::type_system::type_handle::NonGenericTypeHandle {
        type $TMethodId = ::stdlib_header::System::$id::MethodId;
        type $TStaticMethodId = ::stdlib_header::System::$id::StaticMethodId;
        $crate::stdlib::System::define_struct(
            ::stdlib_header::CoreTypeId::${concat(System_, $id)},
            |#[allow(unused)] $mt, #[allow(unused)] $method_info| match unsafe { $method_info.get_id::<$TMethodId>() } {
                $(
                    $TMethodId::$MethodName => $f,
                )*
                $TMethodId::__END => unreachable!(),
            },
            |#[allow(unused)] $mt, #[allow(unused)] $method_info| match unsafe { $method_info.get_id::<$TStaticMethodId>() } {
                $(
                    $TStaticMethodId::$StaticMethodName => $static_f,
                )*
            },
        )($assembly)
    }
}

pub fn define_class(
    id: CoreTypeId,
    get_method: impl Fn(NonNull<MethodTable<Class>>, MethodInfo) -> Box<Method<Class>>,
    get_static_method: impl Fn(NonNull<MethodTable<Class>>, MethodInfo) -> Box<Method<Class>>,
) -> impl FnOnce(&Assembly) -> NonGenericTypeHandle {
    move |assembly| {
        let CoreTypeInfo {
            id: _,
            kind,
            attr,
            name,
            // TODO: impl it
            generic_count: _,
            parent,
            parent_generics,
            methods,
            static_methods,
            fields,
        } = id.get_core_type_info()();
        debug_assert_eq!(kind, CoreTypeKind::Class);
        NonGenericTypeHandle::Class(
            Class::new(
                NonNull::from_ref(assembly),
                name,
                attr,
                parent.map(|x| match x {
                    stdlib_header::CoreTypeRef::Core(core_type_id) => {
                        *assembly.get_class(core_type_id as _).unwrap().unwrap()
                    }
                    _ => panic!("Unsupported parent"),
                }),
                parent_generics
                    .into_iter()
                    .map(MaybeUnloadedTypeHandle::from)
                    .collect(),
                MethodTable::wrap_as_method_generator(|mt| {
                    methods
                        .into_iter()
                        .map(|x| get_method(mt, x))
                        .chain(static_methods.into_iter().map(|x| get_static_method(mt, x)))
                        .collect()
                }),
                fields.into_iter().map(map_field_info).collect(),
                None,
                None,
            )
            .as_non_null_ptr(),
        )
    }
}

pub fn define_struct(
    id: CoreTypeId,
    get_method: impl Fn(NonNull<MethodTable<Struct>>, MethodInfo) -> Box<Method<Struct>>,
    get_static_method: impl Fn(NonNull<MethodTable<Struct>>, MethodInfo) -> Box<Method<Struct>>,
) -> impl FnOnce(&Assembly) -> NonGenericTypeHandle {
    move |assembly| {
        let CoreTypeInfo {
            id: _,
            kind,
            attr,
            name,
            // TODO: impl it
            generic_count: _,
            parent,
            parent_generics,
            methods,
            static_methods,
            fields,
        } = id.get_core_type_info()();
        debug_assert!(parent.is_none());
        debug_assert!(parent_generics.is_empty());
        debug_assert_eq!(kind, CoreTypeKind::Struct);
        NonGenericTypeHandle::Struct(
            Struct::new(
                NonNull::from_ref(assembly),
                name,
                attr,
                MethodTable::wrap_as_method_generator(|mt| {
                    methods
                        .into_iter()
                        .map(|x| get_method(mt, x))
                        .chain(static_methods.into_iter().map(|x| get_static_method(mt, x)))
                        .collect()
                }),
                fields.into_iter().map(map_field_info).collect(),
                None,
                None,
            )
            .as_non_null_ptr(),
        )
    }
}
