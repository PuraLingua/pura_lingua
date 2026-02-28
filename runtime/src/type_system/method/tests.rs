#![allow(nonstandard_style, unused)]

use global::string_name;

use crate::{
    stdlib::{
        CoreTypeId, System_Exception_MethodId, System_Object_MethodId,
        System_Object_StaticMethodId, System_String_MethodId, System_UInt8_MethodId,
        System_UInt8_StaticMethodId, System_UInt64_MethodId, System_UInt64_StaticMethodId,
    },
    test_utils::{g_core_class, g_core_type},
    type_system::{
        assembly::Assembly, assembly_manager::AssemblyRef, class::Class, field::Field,
        type_ref::TypeRef,
    },
    value::managed_reference::{ArrayAccessor, ManagedReference, StringAccessor},
    virtual_machine::{EnsureVirtualMachineInitialized, cpu::MainResult, global_vm},
};

use super::*;

#[test]
fn test_call() {
    EnsureVirtualMachineInitialized();

    let vm = global_vm();
    let cpu_id = vm.add_cpu();
    let cpu = vm.get_cpu(cpu_id).unwrap();
    let assembly_manager = vm.assembly_manager();
    assembly_manager.add_assembly(Assembly::new_for_adding(
        "Test".to_owned(),
        false,
        |assembly| vec![],
    ));

    let u8_t = assembly_manager
        .get_core_type(CoreTypeId::System_UInt8)
        .unwrap_struct();
    let u8_t_mt = unsafe { u8_t.as_ref().method_table_ref() };

    let u8_ToString = u8_t_mt
        .get_method(System_UInt8_StaticMethodId::ToString as u32)
        .unwrap();

    let u8_v = 10u8;
    let u8_v_r = &u8_v;
    let ret = unsafe { u8_ToString.as_ref() }.typed_res_call::<ManagedReference<Class>>(
        &cpu,
        None,
        &[(&raw const u8_v_r).cast::<c_void>().cast_mut()],
    );
    let ret_s = unsafe {
        ret.access::<StringAccessor>()
            .unwrap()
            .to_string_lossy()
            .unwrap()
    };
    dbg!(&ret_s);

    let string_t = assembly_manager
        .get_core_type(CoreTypeId::System_String)
        .unwrap_class();
    let string_t_mt = unsafe { string_t.as_ref().method_table_ref() };
    let string_ToString = string_t_mt
        .get_method(System_String_MethodId::ToString as u32)
        .unwrap();
    let ret2 = unsafe { string_ToString.as_ref() }.typed_call::<ManagedReference<Class>>(
        &cpu,
        Some(NonNull::from_ref(&ret).cast()),
        &[],
    );
    let ret2_s = unsafe {
        ret2.access::<StringAccessor>()
            .unwrap()
            .to_string_lossy()
            .unwrap()
    };
    dbg!(&ret2_s);
}

#[test]
fn test_normal_f() {
    EnsureVirtualMachineInitialized();

    let vm = global_vm();
    let cpu_id = vm.add_cpu();
    let cpu = vm.get_cpu(cpu_id).unwrap();
    let assembly_manager = vm.assembly_manager();
    assembly_manager
        .load_binaries(&[
            binary::assembly::Assembly::from_path("../TestData/TestNormalF.plb").unwrap(),
        ])
        .unwrap();

    let assembly = global_vm()
        .assembly_manager()
        .get_assembly_by_name("TestNormalF")
        .unwrap()
        .unwrap();

    let class = assembly.get_class(0).unwrap().unwrap();
    let obj = ManagedReference::<Class>::common_alloc(
        &cpu,
        unsafe { *class.as_ref().method_table() },
        false,
    );

    let f2_id = unsafe {
        dbg!(
            class
                .as_ref()
                .method_table_ref()
                .find_last_method_by_name_ret_id("F2")
                .unwrap()
        )
    };

    let f2 = unsafe { class.as_ref().get_method(f2_id).unwrap() };

    unsafe {
        assert_eq!(
            cpu.invoke_main(f2.as_ref(), vec![]),
            MainResult::VoidWithException
        );
    }
}
