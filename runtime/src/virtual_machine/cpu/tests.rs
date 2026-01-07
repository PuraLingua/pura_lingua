use std::alloc::Layout;

use global::attrs::CallConvention;

use crate::{
    stdlib::CoreTypeId,
    test_utils::g_core_type,
    type_system::{
        assembly::Assembly,
        class::Class,
        method::Method,
        method_table::MethodTable,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    virtual_machine::{EnsureVirtualMachineInitialized, global_vm},
};

use super::*;

#[test]
fn test_call_stack() {
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
                        Some(
                            global_vm()
                                .assembly_manager()
                                .get_core_type(CoreTypeId::System_Object)
                                .unwrap_class(),
                        ),
                        |class| {
                            MethodTable::new(class, |mt| {
                                vec![
                                    Box::new(Method::new(
                                        mt,
                                        "F".to_owned(),
                                        global::attr!(
                                            method Public {}
                                            g_core_type!(System_UInt64),
                                            g_core_type!(System_UInt8),
                                            g_core_type!(System_UInt32),
                                            g_core_type!(System_UInt16),
                                        ),
                                        vec![],
                                        MaybeUnloadedTypeHandle::from(
                                            CoreTypeId::System_Void.global_type_handle(),
                                        ),
                                        CallConvention::PlatformDefault,
                                        None,
                                        vec![],
                                    )),
                                    // Statics
                                    Box::new(Method::default_sctor(
                                        Some(mt),
                                        global::attr!(method Public {}),
                                    )),
                                ]
                            })
                            .as_non_null_ptr()
                        },
                        vec![],
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

    let method = unsafe {
        test_class
            .as_ref()
            .method_table_ref()
            .find_first_method_by_name("F")
            .unwrap()
    };
    unsafe { dbg!(method.as_ref().attr().local_variable_types().len()) };

    let cpu_id = global_vm().add_cpu();
    let cpu = global_vm().get_cpu(cpu_id).unwrap();
    cpu.prepare_call_stack_for_method(unsafe { method.as_ref() })
        .unwrap();

    let call_frame = cpu.current_common_call_frame().unwrap().unwrap();
    let (u64_p, u64_l) = call_frame.get(0).unwrap();
    let (u8_p, u8_l) = call_frame.get(1).unwrap();
    let (u32_p, u32_l) = call_frame.get(2).unwrap();
    let (u16_p, u16_l) = call_frame.get(3).unwrap();
    assert_eq!(Layout::new::<u64>(), u64_l);
    assert_eq!(Layout::new::<u8>(), u8_l);
    assert_eq!(Layout::new::<u32>(), u32_l);
    assert_eq!(Layout::new::<u16>(), u16_l);
    unsafe {
        u64_p.cast::<u64>().write(u64::MAX);
        u8_p.cast::<u8>().write(u8::MAX - 1);
        u32_p.cast::<u32>().write(u32::MAX - 2);
        u16_p.cast::<u16>().write(u16::MAX - 3);
    }

    dbg!(call_frame.as_slice());
}
