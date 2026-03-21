use std::{fmt::Display, ptr::NonNull};

use stdlib_header::CoreTypeId;

use crate::{
    stdlib::{CoreTypeIdConstExt as _, System::define_struct},
    type_system::{
        assembly::Assembly,
        class::Class,
        method::{Method, Parameter},
        method_table::MethodTable,
        r#struct::Struct,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn ToString<T: Display>(
    cpu: &mut CPU,
    _: &Method<Struct>,
    this: &T,
) -> ManagedReference<Class> {
    ManagedReference::new_string(cpu, &this.to_string())
}

macro define(
$assembly:ident, $mt:ident, $method_info:ident, $RustT:ident $(,)?:
    $(
        $Name:ident => $inner:ident;
    )*
@StaticConstructor => $StaticConstructor:expr;
@ToString => $ToString:expr;
) {
    $(
        pub fn $Name(assembly: &Assembly) -> NonGenericTypeHandle {
            fn __get_int_static_methods(
                #[allow(unused)]
                $assembly: &Assembly,
                #[allow(unused)]
                $mt: NonNull<MethodTable<Struct>>,
                #[allow(unused)]
                $method_info: stdlib_header::MethodInfo,
            ) -> Box<Method<Struct>> {
                type $RustT = $inner;
                match unsafe {
                    std::mem::transmute::<_, stdlib_header::definitions::${concat($Name, _StaticMethodId)}>($method_info.id)
                } {
                    stdlib_header::definitions::${concat($Name, _StaticMethodId)}::StaticConstructor => $StaticConstructor,
                    stdlib_header::definitions::${concat($Name, _StaticMethodId)}::ToString => $ToString,
                }
            }
            define_struct(
                CoreTypeId::$Name,
                |_mt, method_info| match unsafe {
                    std::mem::transmute::<_, stdlib_header::definitions::${concat($Name, _MethodId)}>(method_info.id)
                } {
                    stdlib_header::definitions::${concat($Name, _MethodId)}::__END => unreachable!(),
                },
                |mt, method_info| __get_int_static_methods(assembly, mt, method_info),
            )(assembly)
        }
    )*
}

define! {
assembly, mt, method_info, RustT:
    System_UInt8 => u8;
    System_UInt16 => u16;
    System_UInt32 => u32;
    System_UInt64 => u64;
    System_USize => usize;

    System_Int8 => i8;
    System_Int16 => i16;
    System_Int32 => i32;
    System_Int64 => i64;
    System_ISize => isize;
@StaticConstructor =>
    Box::new(Method::default_sctor(
        Some(mt),
        global::attr!(
            method Public {Static}
        ),
    ));
@ToString =>
    Box::new(Method::native(
        Some(mt),
        "ToString".to_owned(),
        global::attr!(
            method Public {Static}
        ),
        vec![Parameter::new(
            MaybeUnloadedTypeHandle::Unloaded(CoreTypeId::System_UInt8.static_type_ref()),
            global::attr!(parameter { ByRef }),
        )],
        MaybeUnloadedTypeHandle::Unloaded(CoreTypeId::System_String.static_type_ref()),
        global::attrs::CallConvention::PlatformDefault,
        None,
        ToString::<RustT> as _,
    ));
}
