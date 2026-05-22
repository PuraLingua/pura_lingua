pub mod AssemblyInfo;
pub mod FieldInfo;
pub mod MethodInfo;
pub mod ParameterInfo;
pub mod TypeInfo;

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
    impl From<::stdlib_header::System::Reflection::$id::MethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::Reflection::$id::MethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    impl From<::stdlib_header::System::Reflection::$id::StaticMethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::Reflection::$id::StaticMethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    pub fn $load(
        $assembly: &$crate::type_system::assembly::Assembly,
    )
    -> $crate::type_system::assembly::TypeContainer {
        type $TMethodId = ::stdlib_header::System::Reflection::$id::MethodId;
        type $TStaticMethodId = ::stdlib_header::System::Reflection::$id::StaticMethodId;
        $crate::stdlib::System::define_class(
            ::stdlib_header::CoreTypeId::${concat(System_Reflection_, $id)},
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
    impl From<::stdlib_header::System::Reflection::$id::MethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::Reflection::$id::MethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    impl From<::stdlib_header::System::Reflection::$id::StaticMethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::Reflection::$id::StaticMethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    pub fn $load(
        $assembly: &$crate::type_system::assembly::Assembly,
    )
    -> $crate::type_system::assembly::TypeContainer {
        type $TMethodId = ::stdlib_header::System::Reflection::$id::MethodId;
        type $TStaticMethodId = ::stdlib_header::System::Reflection::$id::StaticMethodId;
        $crate::stdlib::System::define_struct(
            ::stdlib_header::CoreTypeId::${concat(System_Reflection_, $id)},
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

macro _define_interface(
    fn $load:ident($assembly:ident, $mt:ident, $method_info:ident)
    $id:ident
#methods($TMethodId:ident):
    $(
        $MethodName:ident => $f:expr;
    )*
) {
    impl From<::stdlib_header::System::Reflection::$id::MethodId> for $crate::type_system::method::MethodRef {
        fn from(value: ::stdlib_header::System::Reflection::$id::MethodId) -> Self {
            Self::Index(value as u32)
        }
    }
    pub fn $load(
        $assembly: &$crate::type_system::assembly::Assembly,
    )
    -> $crate::type_system::assembly::TypeContainer {
        type $TMethodId = ::stdlib_header::System::Reflection::$id::MethodId;
        $crate::stdlib::System::define_interface(
            ::stdlib_header::CoreTypeId::${concat(System_Reflection_, $id)},
            |#[allow(unused)] $mt, #[allow(unused)] $method_info| match unsafe { $method_info.get_id::<$TMethodId>() } {
                $(
                    $TMethodId::$MethodName => $f,
                )*
                $TMethodId::__END => unreachable!(),
            },
        )($assembly)
    }
}
