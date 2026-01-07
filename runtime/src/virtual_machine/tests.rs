use global::{attrs::CallConvention, instruction::Instruction, string_name};

use crate::{
    test_utils::{g_core_class, g_core_type},
    type_system::{
        assembly::Assembly, field::Field, method::Method, method_table::MethodTable,
        type_handle::MaybeUnloadedTypeHandle, type_ref::TypeRef,
    },
};

use super::*;

#[test]
fn test_static() {
    EnsureVirtualMachineInitialized();

    global_vm()
        .assembly_manager()
        .add_assembly(Assembly::new_for_adding(
            "Test".to_owned(),
            false,
            |assembly| {
                vec![NonGenericTypeHandle::Class(
                    Class::new(
                        assembly,
                        "Test.Test".to_owned(),
                        global::attr!(
                            class Public {}
                        ),
                        Some(g_core_class!(System_Object)),
                        |class| {
                            MethodTable::new(class, |mt| {
                                // Statics
                                vec![Box::new(Method::new(
                                    mt,
                                    ".sctor".to_owned(),
                                    global::attr!(
                                        method Public {Static}
                                        g_core_type!(System_UInt64),
                                        g_core_type!(System_UInt8),
                                        g_core_type!(System_UInt32),
                                        g_core_type!(System_UInt16),
                                    ),
                                    vec![],
                                    g_core_type!(System_Void),
                                    CallConvention::PlatformDefault,
                                    None,
                                    vec![
                                        Instruction::Load_u64 {
                                            register_addr: 0,
                                            val: 10,
                                        },
                                        Instruction::SetStaticField {
                                            val_addr: 0,
                                            ty: TypeRef::Index {
                                                assembly: string_name!("Test"),
                                                ind: 0,
                                            }
                                            .into(),
                                            field: 0,
                                        },
                                    ],
                                ))]
                            })
                            .as_non_null_ptr()
                        },
                        vec![Field::new(
                            "A".to_owned(),
                            global::attr!(field Public {Static}),
                            g_core_type!(System_UInt64),
                        )],
                        None,
                        None,
                    )
                    .as_non_null_ptr(),
                )]
            },
        ));

    let test_assembly = global_vm()
        .assembly_manager()
        .get_assembly_by_name("Test")
        .unwrap()
        .unwrap();

    let test_class = test_assembly
        .get_type::<NonNull<Class>>(0)
        .unwrap()
        .unwrap();

    let s_field = global_vm()
        .get_static_field((*test_class).into(), 0)
        .unwrap();
    dbg!(unsafe { s_field.0.cast::<u64>().read() }, s_field.1);
}
