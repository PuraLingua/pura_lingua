use std::ptr::NonNull;

use global::{
    attrs::CallConvention,
    instruction::{Instruction, RegisterAddr},
    non_purus_call_configuration, string_name,
};
use stdlib_header::CoreTypeId;

use crate::{
    stdlib::{
        System_Array_1_MethodId, System_DynamicLibrary_MethodId,
        System_NonPurusCallConfiguration_MethodId, System_NonPurusCallType_StaticMethodId,
    },
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
    virtual_machine::global_vm,
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
                                    Instruction::Load_String {
                                        register_addr: RegisterAddr::new(0),
                                        val: DLL_PATH.to_owned(),
                                    },
                                    Instruction::NewObject {
                                        ty: g_core_type!(System_DynamicLibrary),
                                        ctor_name:
                                            System_DynamicLibrary_MethodId::Constructor_String
                                                .into(),
                                        args: vec![RegisterAddr::new(0)],
                                        register_addr: RegisterAddr::new(1),
                                    },
                                    Instruction::SetStaticField {
                                        val_addr: RegisterAddr::new(1),
                                        ty: TEST_CLASS_REF.into(),
                                        field: 0,
                                    },
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

    let cpu_id = vm.add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

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
                    .typed_res_call::<libc::time_t>(&cpu, None, &[])
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
                    .typed_res_call::<windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT>(&cpu, None, &[])
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
            Instruction::LoadStatic {
                register_addr: RegisterAddr::new(17),
                ty: TEST_CLASS_REF.into(),
                field: 0,
            },
            // LoadMethod
            Instruction::Load_String {
                register_addr: RegisterAddr::new(18),
                val: "MessageBoxW".to_owned(),
            },
            Instruction::InstanceCall {
                val: RegisterAddr::new(17),
                method: MethodRef::from(System_DynamicLibrary_MethodId::GetSymbol),
                args: vec![RegisterAddr::new(18)],
                ret_at: RegisterAddr::new(0),
            },
            /* #region Arguments */
            Instruction::Load_u64 {
                register_addr: RegisterAddr::new(1),
                val: 0,
            },
            Instruction::Load_String {
                register_addr: RegisterAddr::new(2),
                val: "Test passed".to_owned(),
            },
            Instruction::Load_String {
                register_addr: RegisterAddr::new(3),
                val: "INFO".to_owned(),
            },
            Instruction::Load_u32 {
                register_addr: RegisterAddr::new(4),
                // cSpell:disable-next-line
                val: windows::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION.0,
            },
            /* #endregion */

            /* #region Config Setup */
            // Call convention
            Instruction::Load_u8 {
                register_addr: RegisterAddr::new(6),
                val: CallConvention::PlatformDefault.into(),
            },
            // Return type
            Instruction::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateI32.into(),
                args: vec![],
                ret_at: RegisterAddr::new(7),
            },
            // Encoding
            Instruction::Load_u8 {
                register_addr: RegisterAddr::new(8),
                val: non_purus_call_configuration::StringEncoding::C_Utf16.into(),
            },
            // Object strategy
            Instruction::Load_u8 {
                register_addr: RegisterAddr::new(9),
                val: non_purus_call_configuration::ObjectStrategy::PointToData.into(),
            },
            // New by ref argument array
            Instruction::NewArray {
                element_type: CoreTypeId::System_USize.static_type_ref().into(),
                len: 0,
                register_addr: RegisterAddr::new(10),
            },
            // New argument array
            Instruction::NewArray {
                element_type: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                len: 4,
                register_addr: RegisterAddr::new(11),
            },
            // Arg0
            Instruction::Load_u64 {
                register_addr: RegisterAddr::new(12),
                val: 0,
            },
            Instruction::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreatePointer.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            },
            Instruction::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            },
            // Arg 1
            Instruction::Load_u64 {
                register_addr: RegisterAddr::new(12),
                val: 1,
            },
            Instruction::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateString.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            },
            Instruction::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            },
            // Arg 2
            Instruction::Load_u64 {
                register_addr: RegisterAddr::new(12),
                val: 2,
            },
            Instruction::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateString.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            },
            Instruction::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            },
            // Arg 3
            Instruction::Load_u64 {
                register_addr: RegisterAddr::new(12),
                val: 3,
            },
            Instruction::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateU32.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            },
            Instruction::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            },
            /* #endregion */
            // Construct
            Instruction::NewObject {
                ty: CoreTypeId::System_NonPurusCallConfiguration
                    .static_type_ref()
                    .into(),
                ctor_name: System_NonPurusCallConfiguration_MethodId::Constructor.into(),
                args: vec![
                    RegisterAddr::new(6),
                    RegisterAddr::new(7),
                    RegisterAddr::new(8),
                    RegisterAddr::new(9),
                    RegisterAddr::new(10),
                    RegisterAddr::new(11),
                ],
                register_addr: RegisterAddr::new(5),
            },
            Instruction::DynamicNonPurusCall {
                f_pointer: RegisterAddr::new(0),
                config: RegisterAddr::new(5),
                args: vec![
                    RegisterAddr::new(1),
                    RegisterAddr::new(2),
                    RegisterAddr::new(3),
                    RegisterAddr::new(4),
                ],
                ret_at: RegisterAddr::new(16),
            },
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
            Instruction::LoadStatic {
                register_addr: RegisterAddr::new(0),
                ty: TEST_CLASS_REF.into(),
                field: 0,
            },
            // LoadMethod
            Instruction::Load_String {
                register_addr: RegisterAddr::new(1),
                val: "time".to_owned(),
            },
            Instruction::InstanceCall {
                val: RegisterAddr::new(0),
                method: MethodRef::from(System_DynamicLibrary_MethodId::GetSymbol),
                args: vec![RegisterAddr::new(1)],
                ret_at: RegisterAddr::new(2),
            },
            // Prepare Config

            // CallConvention
            Instruction::Load_u8 {
                register_addr: RegisterAddr::new(4),
                val: CallConvention::PlatformDefault as u8,
            },
            // ReturnType
            Instruction::StaticCall {
                ty: assembly_manager
                    .get_core_type(CoreTypeId::System_NonPurusCallType)
                    .into(),
                method: System_NonPurusCallType_StaticMethodId::CreateI64.into(),
                args: vec![],
                ret_at: RegisterAddr::new(5),
            },
            // Encoding
            Instruction::Load_u8 {
                register_addr: RegisterAddr::new(6),
                val: non_purus_call_configuration::StringEncoding::C_Utf16 as _,
            },
            // ObjectStrategy
            Instruction::Load_u8 {
                register_addr: RegisterAddr::new(7),
                val: non_purus_call_configuration::ObjectStrategy::PointToData as _,
            },
            // ByRefArguments
            Instruction::NewArray {
                element_type: assembly_manager
                    .get_core_type(CoreTypeId::System_USize)
                    .into(),
                len: 0,
                register_addr: RegisterAddr::new(8),
            },
            // Arguments
            Instruction::NewArray {
                element_type: assembly_manager
                    .get_core_type(CoreTypeId::System_USize)
                    .into(),
                len: 1,
                register_addr: RegisterAddr::new(9),
            },
            // IndexToSet
            Instruction::Load_u64 {
                register_addr: RegisterAddr::new(10),
                val: 0,
            },
            // Arg0
            Instruction::StaticCall {
                ty: assembly_manager
                    .get_core_type(CoreTypeId::System_NonPurusCallType)
                    .into(),
                method: System_NonPurusCallType_StaticMethodId::CreatePointer.into(),
                args: vec![],
                ret_at: RegisterAddr::new(11),
            },
            Instruction::InstanceCall {
                val: RegisterAddr::new(9),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(10), RegisterAddr::new(11)],
                ret_at: RegisterAddr::new(0),
            },
            // Construct
            Instruction::NewObject {
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
                register_addr: RegisterAddr::new(3),
            },
            // Call
            Instruction::Load_u64 {
                register_addr: RegisterAddr::new(12),
                val: 0,
            },
            Instruction::DynamicNonPurusCall {
                f_pointer: RegisterAddr::new(2),
                config: RegisterAddr::new(3),
                args: vec![RegisterAddr::new(12)],
                ret_at: RegisterAddr::new(13),
            },
            Instruction::ReturnVal {
                register_addr: RegisterAddr::new(13),
            },
        ],
    ))
}
