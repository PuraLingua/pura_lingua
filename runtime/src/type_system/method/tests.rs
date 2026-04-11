#![allow(nonstandard_style, unused)]

use global::{
    instruction::{
        IRegisterAddr, Instruction_Call, Instruction_Load, Instruction_New, LoadContent,
        ShortRegisterAddr,
    },
    string_name,
};

use crate::{
    stdlib::CoreTypeId,
    test_utils::{g_core_class, g_core_type},
    type_system::{
        assembly::Assembly,
        assembly_manager::AssemblyRef,
        class::Class,
        field::Field,
        interface::{Interface, InterfaceImplementation},
        type_ref::TypeRef,
    },
    value::managed_reference::{ArrayAccessor, ManagedReference, StringAccessor},
    virtual_machine::{CpuID, EnsureGlobalVirtualMachineInitialized, cpu::MainResult, global_vm},
};

use super::*;

#[test]
fn test_call() {
    EnsureGlobalVirtualMachineInitialized();

    let mut cpu = CpuID::new_write_global();
    let assembly_manager = global_vm().assembly_manager();
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
        .get_method(stdlib_header::StaticMethodId!(UInt8::ToString) as u32)
        .unwrap();

    let u8_v = 10u8;
    let u8_v_r = &u8_v;
    let ret = unsafe { u8_ToString.as_ref() }.typed_res_call::<ManagedReference<Class>>(
        &mut cpu,
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
        .get_method(stdlib_header::MethodId!(String::ToString) as u32)
        .unwrap();
    let ret2 = unsafe { string_ToString.as_ref() }.typed_call::<ManagedReference<Class>>(
        &mut cpu,
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
    EnsureGlobalVirtualMachineInitialized();

    let mut cpu = CpuID::new_write_global();
    let assembly_manager = global_vm().assembly_manager();
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
        &mut cpu,
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

#[test]
fn test_interface_call() {
    extern "system" fn Println(cpu: &CPU, method: &Method<Class>, val: ManagedReference<Class>) {
        println!(
            "{}",
            val.access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap()
                .display()
        );
    }

    let assembly = crate::test_utils::new_global_assembly("Test", |assembly| {
        vec![
            Interface::new(
                assembly,
                "Test::ITest".to_owned(),
                global::attr!(
                    interface Public {}
                ),
                vec![],
                MethodTable::wrap_as_method_generator(|mt| {
                    vec![Box::new(Method::new(
                        mt,
                        "FTest".to_owned(),
                        global::attr!(
                            method Public {}
                        ),
                        vec![],
                        g_core_type!(System_String),
                        Default::default(),
                        None,
                        vec![],
                    ))]
                }),
                None,
            )
            .into(),
            Class::new(
                assembly,
                "Test::Test1".to_owned(),
                global::attr!(class Public {}),
                Some(g_core_class!(System_Object)),
                vec![],
                MethodTable::wrap_as_method_generator(|mt| {
                    vec![
                        Box::new(Method::new(
                            mt,
                            ".ctor".to_owned(),
                            global::attr!(
                                method Public {}
                            ),
                            vec![],
                            g_core_type!(System_Void),
                            Default::default(),
                            None,
                            vec![],
                        )),
                        Box::new(Method::new(
                            mt,
                            "AAA".to_owned(),
                            global::attr!(
                                method Public {}
                            ),
                            vec![],
                            g_core_type!(System_Void),
                            Default::default(),
                            None,
                            vec![],
                        )),
                        Box::new(Method::new(
                            mt,
                            "FTest".to_owned(),
                            global::attr!(
                                method Public {}
                                g_core_type!(System_String)
                            ),
                            vec![],
                            g_core_type!(System_String),
                            Default::default(),
                            None,
                            vec![
                                Instruction::SLoad(Instruction_Load {
                                    addr: ShortRegisterAddr::new(0),
                                    content: LoadContent::String("From Test1".to_owned()),
                                }),
                                Instruction::SReturnVal {
                                    register_addr: ShortRegisterAddr::new(0),
                                },
                            ],
                        )),
                        // Statics
                        Box::new(Method::default_sctor(
                            Some(mt),
                            global::attr!(method Public {Static}),
                        )),
                    ]
                }),
                vec![],
                None,
                vec![InterfaceImplementation {
                    target: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                        assembly: AssemblyRef::Name(string_name!("Test")),
                        ind: 0,
                    }),
                    map: vec![stdlib_header::System::Object::MethodId::__END as u32 + 2],
                }],
                None,
            )
            .into(),
            Class::new(
                assembly,
                "Test::Test2".to_owned(),
                global::attr!(class Public {}),
                Some(g_core_class!(System_Object)),
                vec![],
                MethodTable::wrap_as_method_generator(|mt| {
                    vec![
                        Box::new(Method::new(
                            mt,
                            ".ctor".to_owned(),
                            global::attr!(
                                method Public {}
                            ),
                            vec![],
                            g_core_type!(System_Void),
                            Default::default(),
                            None,
                            vec![],
                        )),
                        Box::new(Method::new(
                            mt,
                            "FTest".to_owned(),
                            global::attr!(
                                method Public {}
                                g_core_type!(System_String)
                            ),
                            vec![],
                            g_core_type!(System_String),
                            Default::default(),
                            None,
                            vec![
                                Instruction::SLoad(Instruction_Load {
                                    addr: ShortRegisterAddr::new(0),
                                    content: LoadContent::String("From Test2".to_owned()),
                                }),
                                Instruction::SReturnVal {
                                    register_addr: ShortRegisterAddr::new(0),
                                },
                            ],
                        )),
                        // Statics
                        Box::new(Method::default_sctor(
                            Some(mt),
                            global::attr!(method Public {Static}),
                        )),
                    ]
                }),
                vec![],
                None,
                vec![InterfaceImplementation {
                    target: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                        assembly: AssemblyRef::Name(string_name!("Test")),
                        ind: 0,
                    }),
                    map: vec![stdlib_header::System::Object::MethodId::__END as u32 + 1],
                }],
                None,
            )
            .into(),
            Class::new(
                assembly,
                "Test::Main".to_owned(),
                global::attr!(class Public {}),
                Some(g_core_class!(System_Object)),
                vec![],
                MethodTable::wrap_as_method_generator(|mt| {
                    vec![
                        Box::new(Method::new(
                            mt,
                            "Main".to_owned(),
                            global::attr!(
                                method Public {Static}
                                MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                    assembly: AssemblyRef::Name(string_name!("Test")),
                                    ind: 0,
                                }),
                                MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                    assembly: AssemblyRef::Name(string_name!("Test")),
                                    ind: 0,
                                }),
                                g_core_type!(System_String),
                                g_core_type!(System_Void),
                            ),
                            vec![],
                            g_core_type!(System_Void),
                            Default::default(),
                            None,
                            vec![
                                Instruction::SNew(Instruction_New::NewObject {
                                    ty: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                        assembly: AssemblyRef::Name(string_name!("Test")),
                                        ind: 1,
                                    }),
                                    ctor_name: stdlib_header::System::Object::MethodId::__END
                                        .into(),
                                    args: vec![],
                                    output: ShortRegisterAddr::new(0),
                                }),
                                Instruction::SNew(Instruction_New::NewObject {
                                    ty: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                        assembly: AssemblyRef::Name(string_name!("Test")),
                                        ind: 2,
                                    }),
                                    ctor_name: stdlib_header::System::Object::MethodId::__END
                                        .into(),
                                    args: vec![],
                                    output: ShortRegisterAddr::new(1),
                                }),
                                Instruction::SCall(Instruction_Call::InterfaceCall {
                                    interface: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                        assembly: AssemblyRef::Name(string_name!("Test")),
                                        ind: 0,
                                    }),
                                    val: ShortRegisterAddr::new(0),
                                    method: MethodRef::Index(0),
                                    args: vec![],
                                    ret_at: ShortRegisterAddr::new(2),
                                }),
                                Instruction::SCall(Instruction_Call::StaticCall {
                                    ty: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                        assembly: AssemblyRef::Name(string_name!("Test")),
                                        ind: 3,
                                    }),
                                    method: MethodRef::Index(
                                        stdlib_header::System::Object::MethodId::__END as u32 + 1,
                                    ),
                                    args: vec![ShortRegisterAddr::new(2)],
                                    ret_at: ShortRegisterAddr::new(3),
                                }),
                                Instruction::SCall(Instruction_Call::InterfaceCall {
                                    interface: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                        assembly: AssemblyRef::Name(string_name!("Test")),
                                        ind: 0,
                                    }),
                                    val: ShortRegisterAddr::new(1),
                                    method: MethodRef::Index(0),
                                    args: vec![],
                                    ret_at: ShortRegisterAddr::new(2),
                                }),
                                Instruction::SCall(Instruction_Call::StaticCall {
                                    ty: MaybeUnloadedTypeHandle::Unloaded(TypeRef::Index {
                                        assembly: AssemblyRef::Name(string_name!("Test")),
                                        ind: 3,
                                    }),
                                    method: MethodRef::Index(
                                        stdlib_header::System::Object::MethodId::__END as u32 + 1,
                                    ),
                                    args: vec![ShortRegisterAddr::new(2)],
                                    ret_at: ShortRegisterAddr::new(3),
                                }),
                            ],
                        )),
                        Box::new(Method::native(
                            Some(mt),
                            "Println".to_owned(),
                            global::attr!(method Public {Static}),
                            vec![Parameter::new(
                                g_core_type!(System_String),
                                global::attr!(parameter {}),
                            )],
                            g_core_type!(System_Void),
                            Default::default(),
                            None,
                            Println as _,
                        )),
                        Box::new(Method::default_sctor(
                            Some(mt),
                            global::attr!(method Public {Static}),
                        )),
                    ]
                }),
                vec![],
                None,
                vec![],
                None,
            )
            .into(),
        ]
    });

    let main_class = assembly.get_class(3).unwrap().unwrap();
    let mut main_class = *main_class;
    unsafe {
        *main_class.as_mut().main_mut() =
            Some(stdlib_header::System::Object::MethodId::__END as u32);
    }

    let mut cpu = CpuID::new_write_global();
    assert_eq!(
        cpu.invoke_main_class(unsafe { main_class.as_ref() }, vec![]),
        MainResult::Void
    );
}

#[test]
fn test_interface_from_binary() {
    let mut cpu = CpuID::new_write_global();
    let assembly_manager = global_vm().assembly_manager();
    assembly_manager
        .load_binaries(&[
            binary::assembly::Assembly::from_path("../TestData/SimpleIR.SimpleConsole.plb")
                .unwrap(),
            binary::assembly::Assembly::from_path("../TestData/TestInterface.plb").unwrap(),
        ])
        .unwrap();

    let assembly = global_vm()
        .assembly_manager()
        .get_assembly_by_name("TestInterface")
        .unwrap()
        .unwrap();

    let class = assembly.get_class(3).unwrap().unwrap();
    assert_eq!(
        cpu.invoke_main_class(unsafe { class.as_ref() }, vec![]),
        MainResult::Void
    );
}
