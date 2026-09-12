use global::{
    attrs::CallConvention,
    instruction::{Instruction, Instruction_New, RegisterAddr},
};

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdConstExt},
    test_utils::{core_type_in, new_assembly_on},
    type_system::{
        generics::GenericCountRequirement, method_table::MethodTable,
        type_handle::MaybeUnloadedTypeHandle, type_ref::TypeRef,
    },
    virtual_machine::{create_vm_on_stack, global_vm},
};

use super::*;

#[test]
fn test_to_string() {
    create_vm_on_stack!(vm);

    let mut cpu = vm.add_write_cpu();
    let string_t = vm
        .assembly_manager()
        .get_core_type(CoreTypeId::System_String)
        .unwrap_class();
    let s1 = ManagedReference::new_string(&mut cpu, "aaa");
    let s2 = ManagedReference::new_string(&mut cpu, "bbb");
    let mut arr =
        ManagedReference::alloc_array(&mut cpu, unsafe { *string_t.as_ref().method_table() }, 2);

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
            .get_method(stdlib_header::MethodId!(Object::ToString) as _)
            .unwrap()
    };

    let s = unsafe {
        let arr_r = &arr;
        ToString_m
            .as_ref()
            .typed_res_call::<ManagedReference<Class>>(
                &mut cpu,
                Some(NonNull::from_ref(arr_r).cast()),
                &[],
            )
    };

    assert_eq!(
        s.access::<StringAccessor>().unwrap().get_str().unwrap(),
        widestring::u16cstr!("[aaa, bbb]")
    );
}

#[test]
fn array_get_set() -> global::Result<()> {
    create_vm_on_stack!(vm);

    let assembly = new_assembly_on(&vm, "Test", |assembly| {
        vec![
                    Class::new(
                        assembly,
                        widestring::utf16str!("Test::Test").to_owned(),
                        global::attr!(
                            class Public {}
                        ),
                        GenericCountRequirement::default(),
                        Some(
                            global_vm()
                                .assembly_manager()
                                .get_core_type(CoreTypeId::System_Object)
                                .unwrap_class(),
                        ),
                        vec![],
                        |class| {
                            MethodTable::new(class, |mt| {
                                vec![
                                    Method::new(
                                        mt,
                                        widestring::utf16str!("TestSet").to_owned(),
                                        global::attr!(
                                            method Public {Static}
                                            /* 0 */ MaybeUnloadedTypeHandle::Unloaded(TypeRef::Specific {
                                                assembly_and_index: either::Either::Right(Box::new(
                                                    vm.assembly_manager().get_core_type(CoreTypeId::System_Array_1).into(),
                                                )),
                                                types: vec![core_type_in!(System_String in vm)],
                                            }).into(),
                                            /* 1 */ vm.assembly_manager().get_core_type(CoreTypeId::System_String).into(),
                                            /* 2 */ vm.assembly_manager().get_core_type(CoreTypeId::System_String).into(),
                                            /* 3 */	vm.assembly_manager().get_core_type(CoreTypeId::System_USize).into(),
                                            /* 4 */ vm.assembly_manager().get_core_type(CoreTypeId::System_Void).into(),
                                        ),
                                        GenericCountRequirement::default(),
                                        vec![],
                                        MaybeUnloadedTypeHandle::Unloaded(TypeRef::Specific {
                                            assembly_and_index: either::Either::Right(Box::new(
                                                vm.assembly_manager().get_core_type(CoreTypeId::System_Array_1).into(),
                                            )),
                                            types: vec![vm.assembly_manager().get_core_type(CoreTypeId::System_String).into()],
                                        }).into(),
                                        CallConvention::PlatformDefault,
                                        None,
                                        vec![
                                            Instruction::New(Instruction_New::NewArray {
                                                element_type: CoreTypeId::System_String
                                                    .static_type_ref()
                                                    .into(),
                                                len: 2,
                                                output: RegisterAddr::new(0),
                                            }),
                                            Instruction::Load(Instruction_Load {
                                                addr: RegisterAddr::new(1),
                                                content: LoadContent::String("aaa".to_owned()),
                                            }),
                                            Instruction::Load(Instruction_Load {
                                                addr: RegisterAddr::new(2),
                                                content: LoadContent::String("bbb".to_owned()),
                                            }),

                                            Instruction::Load(Instruction_Load {
                                                addr: RegisterAddr::new(3),
                                                content: LoadContent::U64(0),
                                            }),
                                            Instruction::Call(Instruction_Call::InstanceCall {
                                                val: RegisterAddr::new(0),
                                                method: stdlib_header::MethodId!(Array_1::set_Index).into(),
                                                args: vec![RegisterAddr::new(3), RegisterAddr::new(1)],
                                                ret_at: RegisterAddr::new(4),
                                            }),

                                            Instruction::Load(Instruction_Load {
                                                addr: RegisterAddr::new(3),
                                                content: LoadContent::U64(1),
                                            }),
                                            Instruction::Call(Instruction_Call::InstanceCall {
                                                val: RegisterAddr::new(0),
                                                method: stdlib_header::MethodId!(Array_1::set_Index).into(),
                                                args: vec![RegisterAddr::new(3), RegisterAddr::new(2)],
                                                ret_at: RegisterAddr::new(4),
                                            }),

                                            Instruction::ReturnVal {
                                                register_addr: RegisterAddr::new(0),
                                            }
                                        ],
                                        ExceptionTable::gen_new(),
                                    ),
                                    //statics
                                    Method::default_sctor(Some(mt), global::attr!(method Public {Static})),
                                ]
                            })
                            .into()
                        },
                        vec![],
                        None,
                        vec![],
                        None,
                    )
                    .into(),
                ]
    });

    let mut cpu = vm.add_write_cpu();
    let test_class = assembly.get_class(0).unwrap();
    let m_set = unsafe {
        test_class
            .as_ref()
            .method_table_ref()
            .find_first_method_by_name(widestring::utf16str!("TestSet"))
            .unwrap()
    };
    let arr = unsafe {
        m_set
            .as_ref()
            .typed_res_call::<ManagedReference<Class>>(&mut cpu, None, &[])
    };
    assert!(!arr.is_null());

    let expected_elements = [widestring::u16cstr!("aaa"), widestring::u16cstr!("bbb")];

    for (x, expected) in unsafe {
        arr.access::<ArrayAccessor>()
            .unwrap()
            .as_slice::<ManagedReference<Class>>()
            .unwrap()
            .iter()
            .zip(expected_elements)
    } {
        assert_eq!(
            x.access::<StringAccessor>().unwrap().get_str().unwrap(),
            expected
        );
    }
    Ok(())
}
