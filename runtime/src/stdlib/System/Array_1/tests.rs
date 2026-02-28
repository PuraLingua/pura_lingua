use global::{
    attrs::CallConvention,
    instruction::{Instruction, RegisterAddr},
};

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdConstExt, CoreTypeIdExt as _, System_Array_1_MethodId},
    test_utils::g_core_type,
    type_system::{
        assembly::Assembly, method_table::MethodTable, type_handle::MaybeUnloadedTypeHandle,
        type_ref::TypeRef,
    },
    virtual_machine::{EnsureVirtualMachineInitialized, global_vm},
};

use super::*;

#[test]
fn test_to_string() {
    EnsureVirtualMachineInitialized();

    let cpu_id = global_vm().add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();
    let string_t = CoreTypeId::System_String
        .global_type_handle()
        .unwrap_class();
    let s1 = ManagedReference::new_string(&cpu, "aaa");
    let s2 = ManagedReference::new_string(&cpu, "bbb");
    let mut arr =
        ManagedReference::alloc_array(&cpu, unsafe { *string_t.as_ref().method_table() }, 2);

    unsafe {
        let array_accessor = arr.access_unchecked_mut::<ArrayAccessor>();
        let slice = array_accessor
            .as_slice_mut::<ManagedReference<Class>>()
            .unwrap();
        slice[0] = s1;
        slice[1] = s2;
    }

    let ToString_m = unsafe {
        arr.method_table_ref_unchecked()
            .get_method(System_Object_MethodId::ToString as _)
            .unwrap()
    };

    let s = unsafe {
        let arr_r = &arr;
        ToString_m
            .as_ref()
            .typed_res_call::<ManagedReference<Class>>(
                &cpu,
                Some(NonNull::from_ref(arr_r).cast()),
                &[],
            )
    };

    dbg!(s.access::<StringAccessor>().unwrap().to_string_lossy());
}

#[test]
fn array_get_set() -> global::Result<()> {
    EnsureVirtualMachineInitialized();

    let assembly_id = global_vm()
        .assembly_manager()
        .add_assembly(Assembly::new_for_adding(
            "Test".to_owned(),
            false,
            |assembly| {
                vec![
                    Class::new(
                        assembly,
                        "Test::Test".to_owned(),
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
                                vec![Box::new(Method::new(
                                    mt,
                                    "TestSet".to_owned(),
                                    global::attr!(
                                        method Public {Static}
                                        /* 0 */ MaybeUnloadedTypeHandle::Unloaded(TypeRef::Specific {
											assembly_and_index: either::Either::Right(Box::new(
												g_core_type!(System_String),
											)),
											types: vec![g_core_type!(System_String)],
										}),
                                        /* 1 */ g_core_type!(System_String),
                                        /* 2 */ g_core_type!(System_String),
                                        /* 3 */	g_core_type!(System_USize),
										/* 4 */ g_core_type!(System_Void),
                                    ),
                                    vec![],
                                    MaybeUnloadedTypeHandle::Unloaded(TypeRef::Specific {
                                        assembly_and_index: either::Either::Right(Box::new(
                                            g_core_type!(System_String),
                                        )),
                                        types: vec![g_core_type!(System_String)],
                                    }),
                                    CallConvention::PlatformDefault,
                                    None,
                                    vec![
                                        Instruction::NewArray {
                                            element_type: CoreTypeId::System_String
                                                .static_type_ref()
                                                .into(),
                                            len: 2,
                                            register_addr: RegisterAddr::new(0),
                                        },
                                        Instruction::Load_String {
                                            register_addr: RegisterAddr::new(1),
                                            val: "aaa".to_owned(),
                                        },
                                        Instruction::Load_String {
                                            register_addr: RegisterAddr::new(2),
                                            val: "bbb".to_owned(),
                                        },

										Instruction::Load_u64 {
                                            register_addr: RegisterAddr::new(3),
                                            val: 0,
                                        },
										Instruction::InstanceCall {
											val: RegisterAddr::new(0),
											method: System_Array_1_MethodId::set_Index.into(),
											args: vec![RegisterAddr::new(3), RegisterAddr::new(1)],
											ret_at: RegisterAddr::new(4),
										},

										Instruction::Load_u64 {
                                            register_addr: RegisterAddr::new(3),
                                            val: 1,
                                        },
										Instruction::InstanceCall {
											val: RegisterAddr::new(0),
											method: System_Array_1_MethodId::set_Index.into(),
											args: vec![RegisterAddr::new(3), RegisterAddr::new(2)],
											ret_at: RegisterAddr::new(4),
										},

										Instruction::ReturnVal {
                                            register_addr: RegisterAddr::new(0),
                                        }
                                    ],
                                    )),
                                    //statics
                                    Box::new(Method::default_sctor(Some(mt), global::attr!(method Public {Static}))),
                                ]
                            })
                            .into()
                        },
                        vec![],
                        None,
                        None,
                    )
                    .into(),
                ]
            },
        ));

    let cpu_id = global_vm().add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();
    let assembly = global_vm()
        .assembly_manager()
        .get_assembly(assembly_id)
        .unwrap()
        .unwrap();
    let test_class = assembly.get_class(0).unwrap().unwrap();
    let m_set = unsafe {
        test_class
            .as_ref()
            .method_table_ref()
            .find_first_method_by_name("TestSet")
            .unwrap()
    };
    let arr = unsafe {
        m_set
            .as_ref()
            .typed_res_call::<ManagedReference<Class>>(&cpu, None, &[])
    };
    assert!(!arr.is_null());
    for x in unsafe {
        arr.access::<ArrayAccessor>()
            .unwrap()
            .as_slice::<ManagedReference<Class>>()
            .unwrap()
    } {
        println!(
            "{}",
            x.access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap()
                .display()
        );
    }
    Ok(())
}
