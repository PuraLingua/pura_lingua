use global::{
    attrs::CallConvention,
    instruction::{
        IRegisterAddr, Instruction, Instruction_Load, Instruction_Set, LoadContent, RegisterAddr,
    },
    string_name,
};

use crate::{
    test_utils::{core_class_in, core_type_in, new_assembly_on},
    type_system::{
        assembly::TypeContainer,
        assembly_manager::AssemblyRef,
        field::Field,
        generics::GenericCountRequirement,
        method::{ExceptionTable, Method},
        method_table::MethodTable,
        type_ref::TypeRef,
    },
};

use super::*;

#[test]
fn test_static() {
    create_vm_on_stack!(vm);

    let test_assembly = new_assembly_on(&vm, "Test", |assembly| {
        vec![TypeContainer::from(Class::new(
            assembly,
            widestring::utf16str!("Test::Test").to_owned(),
            global::attr!(
                class Public {}
            ),
            GenericCountRequirement::default(),
            Some(core_class_in!(System_Object in vm)),
            vec![],
            |class| {
                MethodTable::new(class, |mt| {
                    // Statics
                    vec![Method::new(
                        mt,
                        widestring::utf16str!(".sctor").to_owned(),
                        global::attr!(
                            method Public {Static}
                            core_type_in!(System_UInt64 in vm).into(),
                            core_type_in!(System_UInt8 in vm).into(),
                            core_type_in!(System_UInt32 in vm).into(),
                            core_type_in!(System_UInt16 in vm).into(),
                        ),
                        GenericCountRequirement::default(),
                        vec![],
                        core_type_in!(System_Void in vm).into(),
                        CallConvention::PlatformDefault,
                        None,
                        vec![
                            Instruction::Load(Instruction_Load {
                                addr: RegisterAddr::new(0),
                                content: LoadContent::U64(10),
                            }),
                            Instruction::Set(Instruction_Set::Static {
                                val: RegisterAddr::new(0),
                                ty: TypeRef::Index {
                                    assembly: AssemblyRef::Name(string_name!("Test")),
                                    ind: 0,
                                }
                                .into(),
                                field: 0,
                            }),
                        ],
                        ExceptionTable::gen_new(),
                    )]
                })
                .as_non_null_ptr()
            },
            vec![Field::new(
                widestring::utf16str!("A").to_owned(),
                global::attr!(field Public {Static}),
                core_type_in!(System_UInt64 in vm),
            )],
            None,
            vec![],
            None,
        ))]
    });

    let test_class = test_assembly.get_class(0).unwrap();

    let s_field = vm.get_static_field(test_class.into(), 0).unwrap();

    assert_eq!(s_field.1, Layout::from_size_align(8, 8).unwrap());
    assert_eq!(unsafe { s_field.0.cast::<u64>().read() }, 10);
}
