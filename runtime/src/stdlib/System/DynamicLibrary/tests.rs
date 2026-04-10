use std::ptr::NonNull;

use global::{
    attrs::CallConvention,
    instruction::{
        IRegisterAddr, Instruction, Instruction_Load, Instruction_New, Instruction_Set,
        LoadContent, RegisterAddr,
    },
    non_purus_call_configuration, string_name,
};
use stdlib_header::CoreTypeId;

use crate::{
    test_utils::g_core_type,
    type_system::{
        assembly::Assembly,
        assembly_manager::{AssemblyManager, AssemblyRef},
        class::Class,
        field::Field,
        method::{Method, MethodRef},
        method_table::MethodTable,
        type_ref::TypeRef,
    },
    virtual_machine::{CpuID, global_vm},
};

#[test]
fn simple_dynamic_lib_test() {
    let vm = global_vm();

    const DLL_PATH: &str = cfg_select! {
        windows => { "User32.dll" }
        unix => { "/lib/x86_64-linux-gnu/libc.so.6" }
    };

    const TEST_CLASS_REF: TypeRef = TypeRef::Index {
        assembly: AssemblyRef::Name(string_name!("Test")),
        ind: 0,
    };

    let assembly_manager = vm.assembly_manager();
    assembly_manager.add_assembly(Assembly::new_for_adding(
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
                        assembly_manager
                            .get_core_type(CoreTypeId::System_Object)
                            .unwrap_class(),
                    ),
                    vec![],
                    MethodTable::wrap_as_method_generator(|mt| {
                        vec![
                            // Statics
                            Box::new(Method::new(
                                mt,
                                ".sctor".to_owned(),
                                global::attr!(
                                    method Public {Static}
                                    g_core_type!(System_String),
                                    g_core_type!(System_DynamicLibrary),
                                ),
                                vec![],
                                assembly_manager
                                    .get_core_type(CoreTypeId::System_Void)
                                    .into(),
                                CallConvention::PlatformDefault,
                                None,
                                vec![
                                    Instruction::Load(Instruction_Load {
                                        addr: RegisterAddr::new(0),
                                        content: LoadContent::String(DLL_PATH.to_owned()),
                                    }),
                                    Instruction::New(Instruction_New::NewObject {
                                        ty: g_core_type!(System_DynamicLibrary),
                                        ctor_name:
                                            stdlib_header::System::DynamicLibrary::MethodId::Constructor_String
                                                .into(),
                                        args: vec![RegisterAddr::new(0)],
                                        output: RegisterAddr::new(1),
                                    }),
                                    Instruction::Set(Instruction_Set::Static {
                                        val: RegisterAddr::new(1),
                                        ty: TEST_CLASS_REF.into(),
                                        field: 0,
                                    }),
                                ],
                            )),
                            gen_simple_dynamic_lib_to_invoke(assembly_manager, mt),
                        ]
                    }),
                    vec![Field::new(
                        "LIB".to_owned(),
                        global::attr!(field Public {Static}),
                        assembly_manager
                            .get_core_type(CoreTypeId::System_DynamicLibrary)
                            .into(),
                    )],
                    None,
                    None,
                )
                .into(),
            ]
        },
    ));

    let mut cpu = CpuID::new_write_global();

    let assem = global_vm()
        .assembly_manager()
        .get_assembly_by_name("Test")
        .unwrap()
        .unwrap();

    let class = assem.get_class(0).unwrap().unwrap();
    let mt = unsafe { class.as_ref().method_table_ref() };
    let fn_to_invoke = mt.find_first_method_by_name("ToInvoke").unwrap();
    cfg_select! {
        unix => {
            let result = unsafe {
                fn_to_invoke
                    .as_ref()
                    .typed_res_call::<libc::time_t>(&mut cpu, None, &[])
            };
            let mut buffer: [libc::c_char; 80] = [0; 80];
            let time_info = unsafe { libc::localtime(&raw const result) };
            unsafe {
                let len = libc::strftime(buffer.as_mut_ptr(), buffer.len(), c"%Y-%m-%d %H:%M:%S".as_ptr(), time_info.cast_const());
                if len == 0 {
                    panic!("CANNOT FORMAT");
                }
                println!("Current time is: {}", std::ffi::CStr::from_ptr(buffer.as_ptr()).display());
            }
        }
        windows => {
            let result = unsafe {
                fn_to_invoke
                    .as_ref()
                    .typed_res_call::<windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT>(&mut cpu, None, &[])
            };
            println!(
                "You clicked {}",
                if result == windows::Win32::UI::WindowsAndMessaging::IDOK {
                    "OK"
                } else {
                    "<NOTHING>"
                }
            );
        }
    }
}

#[cfg(windows)]
fn gen_simple_dynamic_lib_to_invoke(
    _assembly_manager: &AssemblyManager,
    mt: NonNull<MethodTable<Class>>,
) -> Box<Method<Class>> {
    use global::instruction::Instruction_Call;

    use crate::stdlib::CoreTypeIdConstExt as _;

    const TEST_CLASS_REF: TypeRef = TypeRef::Index {
        assembly: AssemblyRef::Name(string_name!("Test")),
        ind: 0,
    };

    Box::new(Method::new(
        mt,
        "ToInvoke".to_owned(),
        global::attr!(
            method Public {Static}
            /* 0 */ g_core_type!(System_USize), // Pointer to function

            /* 1 */ g_core_type!(System_Pointer),
            /* 2 */ g_core_type!(System_String),
            /* 3 */ g_core_type!(System_String),
            /* 4 */ g_core_type!(System_UInt32),

            /* 5 */ g_core_type!(System_NonPurusCallConfiguration),
            /* 6 */ g_core_type!(System_UInt8), // Call convention
            /* 7 */ g_core_type!(System_NonPurusCallType), // Return type
            /* 8 */ g_core_type!(System_UInt8), // Encoding
            /* 9 */ g_core_type!(System_UInt8), // Object strategy
            /* 10 */ g_core_type!(System_Object), // Array(ByRefArguments)
            /* 11 */ g_core_type!(System_Object), // Array(Arguments)
            /* 12 */ g_core_type!(System_USize), // Index for setting
            /* 13 */ g_core_type!(System_USize), // For 10
            /* 14 */ g_core_type!(System_NonPurusCallType), // For 11
            /* 15 */ g_core_type!(System_Void),

            /* 16 */ g_core_type!(System_Int32), // RET

            /* 17 */ g_core_type!(System_DynamicLibrary), // Library
            /* 18 */ g_core_type!(System_String), // MethodName
        ),
        vec![],
        g_core_type!(System_Int32),
        CallConvention::PlatformDefault,
        None,
        vec![
            // LoadLibrary
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(17),
                content: LoadContent::Static {
                    ty: TEST_CLASS_REF.into(),
                    field: 0,
                },
            }),
            // LoadMethod
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(18),
                content: LoadContent::String("MessageBoxW".to_owned()),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(17),
                method: MethodRef::from(stdlib_header::MethodId!(DynamicLibrary::GetSymbol)),
                args: vec![RegisterAddr::new(18)],
                ret_at: RegisterAddr::new(0),
            }),
            /* #region Arguments */
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(1),
                content: LoadContent::U64(0),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(2),
                content: LoadContent::String("Test passed".to_owned()),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(3),
                content: LoadContent::String("INFO".to_owned()),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(4),
                content: LoadContent::U32(
                    windows::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION.0,
                ),
            }),
            /* #endregion */

            /* #region Config Setup */
            // Call convention
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(6),
                content: LoadContent::U8(CallConvention::PlatformDefault.into()),
            }),
            // Return type
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: stdlib_header::StaticMethodId!(NonPurusCallType::CreateI32).into(),
                args: vec![],
                ret_at: RegisterAddr::new(7),
            }),
            // Encoding
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(8),
                content: LoadContent::U8(
                    non_purus_call_configuration::StringEncoding::C_Utf16.into(),
                ),
            }),
            // Object strategy
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(9),
                content: LoadContent::U8(
                    non_purus_call_configuration::ObjectStrategy::PointToData.into(),
                ),
            }),
            // New by ref argument array
            Instruction::New(Instruction_New::NewArray {
                element_type: CoreTypeId::System_USize.static_type_ref().into(),
                len: 0,
                output: RegisterAddr::new(10),
            }),
            // New argument array
            Instruction::New(Instruction_New::NewArray {
                element_type: CoreTypeId::System_USize.static_type_ref().into(),
                len: 4,
                output: RegisterAddr::new(11),
            }),
            // Arg0
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(12),
                content: LoadContent::U64(0),
            }),
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: stdlib_header::StaticMethodId!(NonPurusCallType::CreatePointer).into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: stdlib_header::MethodId!(Array_1::set_Index).into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            // Arg 1
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(12),
                content: LoadContent::U64(1),
            }),
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: stdlib_header::StaticMethodId!(NonPurusCallType::CreateString).into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: stdlib_header::MethodId!(Array_1::set_Index).into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            // Arg 2
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(12),
                content: LoadContent::U64(2),
            }),
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: stdlib_header::StaticMethodId!(NonPurusCallType::CreateString).into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: stdlib_header::MethodId!(Array_1::set_Index).into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            // Arg 3
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(12),
                content: LoadContent::U64(3),
            }),
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: stdlib_header::StaticMethodId!(NonPurusCallType::CreateU32).into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: stdlib_header::MethodId!(Array_1::set_Index).into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            /* #endregion */
            // Construct
            Instruction::New(Instruction_New::NewObject {
                ty: CoreTypeId::System_NonPurusCallConfiguration
                    .static_type_ref()
                    .into(),
                ctor_name: stdlib_header::MethodId!(NonPurusCallConfiguration::Constructor).into(),
                args: vec![
                    RegisterAddr::new(6),
                    RegisterAddr::new(7),
                    RegisterAddr::new(8),
                    RegisterAddr::new(9),
                    RegisterAddr::new(10),
                    RegisterAddr::new(11),
                ],
                output: RegisterAddr::new(5),
            }),
            Instruction::Call(Instruction_Call::DynamicNonPurusCall {
                f_pointer: RegisterAddr::new(0),
                config: RegisterAddr::new(5),
                args: vec![
                    RegisterAddr::new(1),
                    RegisterAddr::new(2),
                    RegisterAddr::new(3),
                    RegisterAddr::new(4),
                ],
                ret_at: RegisterAddr::new(16),
            }),
            Instruction::ReturnVal {
                register_addr: RegisterAddr::new(16),
            },
        ],
    ))
}

#[cfg(unix)]
fn gen_simple_dynamic_lib_to_invoke(
    assembly_manager: &AssemblyManager,
    mt: NonNull<MethodTable<Class>>,
) -> Box<Method<Class>> {
    use global::instruction::Instruction_Call;
    use stdlib_header::definitions::{
        System_Array_1_MethodId, System_NonPurusCallConfiguration_MethodId,
        System_NonPurusCallType_StaticMethodId,
    };

    const TEST_CLASS_REF: TypeRef = TypeRef::Index {
        assembly: AssemblyRef::Name(string_name!("Test")),
        ind: 0,
    };

    Box::new(Method::new(
        mt,
        "ToInvoke".to_owned(),
        global::attr!(
            method Public {Static}
            /* 0 */ g_core_type!(System_DynamicLibrary), // Library
            /* 1 */ g_core_type!(System_String), // MethodName
            /* 2 */ g_core_type!(System_Pointer), // lpMethod

            // CallConfig
            /* 3 */ g_core_type!(System_NonPurusCallConfiguration), // Config
            /* 4 */ g_core_type!(System_UInt8), // CallConvention

            /* 5 */ g_core_type!(System_NonPurusCallType), // ReturnType

            /* 6 */ g_core_type!(System_UInt8), // Encoding
            /* 7 */ g_core_type!(System_UInt8), // ObjectStrategy
            /* 8 */ g_core_type!(System_Object), // ByRefArguments(System::Array`1[System::USize])

            /* 9 */ g_core_type!(System_Object), // Arguments(System::Array`1[System::NonPurusCallType])
            /* 10 */ g_core_type!(System_USize), // IndexToSet

            // Arg0
            /* 11 */ g_core_type!(System_NonPurusCallType), // Arg0Type

            // Call
            /* 12 */ g_core_type!(System_Pointer), // tLoc
            /* 13 */ g_core_type!(System_Int64), // RET
        ),
        vec![],
        assembly_manager
            .get_core_type(CoreTypeId::System_Int64)
            .into(),
        CallConvention::PlatformDefault,
        None,
        vec![
            // LoadLibrary
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(0),
                content: LoadContent::Static {
                    ty: TEST_CLASS_REF.into(),
                    field: 0,
                },
            }),
            // LoadMethod
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(1),
                content: LoadContent::String("time".to_owned()),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(0),
                method: MethodRef::from(System_DynamicLibrary_MethodId::GetSymbol),
                args: vec![RegisterAddr::new(1)],
                ret_at: RegisterAddr::new(2),
            }),
            // Prepare Config

            // CallConvention
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(4),
                content: LoadContent::U8(CallConvention::PlatformDefault as u8),
            }),
            // ReturnType
            Instruction::Call(Instruction_Call::StaticCall {
                ty: assembly_manager
                    .get_core_type(CoreTypeId::System_NonPurusCallType)
                    .into(),
                method: System_NonPurusCallType_StaticMethodId::CreateI64.into(),
                args: vec![],
                ret_at: RegisterAddr::new(5),
            }),
            // Encoding
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(6),
                content: LoadContent::U8(
                    non_purus_call_configuration::StringEncoding::C_Utf16 as _,
                ),
            }),
            // ObjectStrategy
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(7),
                content: LoadContent::U8(
                    non_purus_call_configuration::ObjectStrategy::PointToData as _,
                ),
            }),
            // ByRefArguments
            Instruction::New(Instruction_New::NewArray {
                element_type: assembly_manager
                    .get_core_type(CoreTypeId::System_USize)
                    .into(),
                len: 0,
                output: RegisterAddr::new(8),
            }),
            // Arguments
            Instruction::New(Instruction_New::NewArray {
                element_type: assembly_manager
                    .get_core_type(CoreTypeId::System_USize)
                    .into(),
                len: 1,
                output: RegisterAddr::new(9),
            }),
            // IndexToSet
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(10),
                content: LoadContent::U8(0),
            }),
            // Arg0
            Instruction::Call(Instruction_Call::StaticCall {
                ty: assembly_manager
                    .get_core_type(CoreTypeId::System_NonPurusCallType)
                    .into(),
                method: System_NonPurusCallType_StaticMethodId::CreatePointer.into(),
                args: vec![],
                ret_at: RegisterAddr::new(11),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(9),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(10), RegisterAddr::new(11)],
                ret_at: RegisterAddr::new(0),
            }),
            // Construct
            Instruction::New(Instruction_New::NewObject {
                ty: assembly_manager
                    .get_core_type(CoreTypeId::System_NonPurusCallConfiguration)
                    .into(),
                ctor_name: System_NonPurusCallConfiguration_MethodId::Constructor.into(),
                args: vec![
                    RegisterAddr::new(4),
                    RegisterAddr::new(5),
                    RegisterAddr::new(6),
                    RegisterAddr::new(7),
                    RegisterAddr::new(8),
                    RegisterAddr::new(9),
                ],
                output: RegisterAddr::new(3),
            }),
            // Call
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(12),
                content: LoadContent::U64(0),
            }),
            Instruction::Call(Instruction_Call::DynamicNonPurusCall {
                f_pointer: RegisterAddr::new(2),
                config: RegisterAddr::new(3),
                args: vec![RegisterAddr::new(12)],
                ret_at: RegisterAddr::new(13),
            }),
            Instruction::ReturnVal {
                register_addr: RegisterAddr::new(13),
            },
        ],
    ))
}
