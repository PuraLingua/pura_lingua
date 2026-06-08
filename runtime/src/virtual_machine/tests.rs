use global::{
    attrs::CallConvention,
    instruction::{
        IRegisterAddr, Instruction, Instruction_Load, Instruction_Set, LoadContent, RegisterAddr,
    },
    string_name,
};

use crate::{
    test_utils::{g_core_class, g_core_type},
    type_system::{
        assembly::{Assembly, TypeContainer},
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
    EnsureGlobalVirtualMachineInitialized();

    global_vm()
        .assembly_manager()
        .add_assembly(Assembly::new_for_adding(
            widestring::utf16str!("Test").to_owned(),
            false,
            |assembly| {
                vec![TypeContainer::from(Class::new(
                    assembly,
                    widestring::utf16str!("Test::Test").to_owned(),
                    global::attr!(
                        class Public {}
                    ),
                    GenericCountRequirement::default(),
                    Some(g_core_class!(System_Object)),
                    vec![],
                    |class| {
                        MethodTable::new(class, |mt| {
                            // Statics
                            vec![Method::new(
                                mt,
                                widestring::utf16str!(".sctor").to_owned(),
                                global::attr!(
                                    method Public {Static}
                                    g_core_type!(System_UInt64).into(),
                                    g_core_type!(System_UInt8).into(),
                                    g_core_type!(System_UInt32).into(),
                                    g_core_type!(System_UInt16).into(),
                                ),
                                GenericCountRequirement::default(),
                                vec![],
                                g_core_type!(System_Void).into(),
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
                        g_core_type!(System_UInt64),
                    )],
                    None,
                    vec![],
                    None,
                ))]
            },
        ));

    let test_assembly = global_vm()
        .assembly_manager()
        .get_assembly_by_name(widestring::utf16str!("Test"))
        .unwrap();

    let test_class = test_assembly.get_class(0).unwrap();

    let s_field = global_vm().get_static_field(test_class.into(), 0).unwrap();

    assert_eq!(s_field.1, Layout::from_size_align(8, 8).unwrap());
    assert_eq!(unsafe { s_field.0.cast::<u64>().read() }, 10);
}
