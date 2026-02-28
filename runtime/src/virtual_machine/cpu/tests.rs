use std::alloc::{Allocator, Layout};

use global::{
    attrs::CallConvention,
    instruction::{Instruction, RegisterAddr},
    non_purus_call_configuration::{ObjectStrategy, StringEncoding},
};
use widestring::U16CStr;

use crate::{
    stdlib::{
        CoreTypeId, CoreTypeIdConstExt, CoreTypeIdExt as _, System_Array_1_MethodId,
        System_NonPurusCallConfiguration_MethodId, System_NonPurusCallType_StaticMethodId,
    },
    test_utils::{LEAK_DETECTOR, g_core_type},
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
    let u64_var = call_frame.get(RegisterAddr::new(0)).unwrap();
    let u8_var = call_frame.get(RegisterAddr::new(1)).unwrap();
    let u32_var = call_frame.get(RegisterAddr::new(2)).unwrap();
    let u16_var = call_frame.get(RegisterAddr::new(3)).unwrap();
    assert_eq!(Layout::new::<u64>(), u64_var.layout);
    assert_eq!(Layout::new::<u8>(), u8_var.layout);
    assert_eq!(Layout::new::<u32>(), u32_var.layout);
    assert_eq!(Layout::new::<u16>(), u16_var.layout);
    unsafe {
        u64_var.ptr.cast::<u64>().write(u64::MAX);
        u8_var.ptr.cast::<u8>().write(u8::MAX - 1);
        u32_var.ptr.cast::<u32>().write(u32::MAX - 2);
        u16_var.ptr.cast::<u16>().write(u16::MAX - 3);
    }

    dbg!(call_frame.as_slice());
}

#[test]
fn static_non_purus_call() {
    extern "system" fn test(a: u64, b: &u32, c: u8, p: *const u16) -> u64 {
        dbg!(a, *b, c);
        println!("{}", unsafe { U16CStr::from_ptr_str(p).display() });
        0
    }
    let f_ptr = test as *const u8;
    let cpu_id = global_vm().add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

    let cfg = NonPurusCallConfiguration {
        call_convention: CallConvention::PlatformDefault,
        return_type: NonPurusCallType::U64,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf16,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![
            (false, NonPurusCallType::U64),
            (true, NonPurusCallType::U32),
            (false, NonPurusCallType::U8),
            (false, NonPurusCallType::String),
        ],
    };
    let a = 0x1ff1u64;
    let b = 10u32;
    let c = 15u8;
    let mut d = ManagedReference::new_string(&cpu, "aaa");
    let (result, result_layout) = cpu.non_purus_call(
        &cfg,
        f_ptr,
        vec![
            (&raw const a).cast_mut().cast(),
            (&raw const b).cast_mut().cast(),
            (&raw const c).cast_mut().cast(),
            (&raw const d).cast_mut().cast(),
        ],
    );
    unsafe {
        dbg!(result.cast::<u64>().as_ref());
    }
    d.destroy(&cpu);

    unsafe {
        LEAK_DETECTOR.deallocate(result, result_layout);
    }
}

#[test]
fn non_purus_call_marshal() {
    extern "system" fn test(a: u64, b: &u32, c: u8, p: *const u16) -> u64 {
        dbg!(a, *b, c);
        println!("{}", unsafe { U16CStr::from_ptr_str(p).display() });
        0
    }
    let f_ptr = test as *const u8;
    let cpu_id = global_vm().add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

    let cfg = NonPurusCallConfiguration {
        call_convention: CallConvention::PlatformDefault,
        return_type: NonPurusCallType::U64,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf16,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![
            (false, NonPurusCallType::U64),
            (true, NonPurusCallType::U32),
            (false, NonPurusCallType::U8),
            (false, NonPurusCallType::String),
        ],
    };
    let marshaled = cpu.marshal_non_purus_configuration(&cfg);
    let cfg_after = cpu.unmarshal_non_purus_configuration(marshaled).unwrap();
    assert_eq!(cfg, cfg_after);
    let a = 0x1ff1u64;
    let b = 10u32;
    let c = 15u8;
    let mut d = ManagedReference::new_string(&cpu, "aaa");
    let (result, result_layout) = cpu.non_purus_call(
        &cfg,
        f_ptr,
        vec![
            (&raw const a).cast_mut().cast(),
            (&raw const b).cast_mut().cast(),
            (&raw const c).cast_mut().cast(),
            (&raw const d).cast_mut().cast(),
        ],
    );
    unsafe {
        dbg!(result.cast::<u64>().as_ref());
    }
    d.destroy(&cpu);

    unsafe {
        LEAK_DETECTOR.deallocate(result, result_layout);
    }
}

#[test]
fn dynamic_non_purus_call() -> global::Result<()> {
    extern "system" fn test(a: u64, b: &u32, c: u8, p: *const u16) -> u64 {
        dbg!(a, *b, c);
        println!("{}", unsafe { U16CStr::from_ptr_str(p).display() });
        0
    }

    global_vm()
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
                                vec![
                                    Box::new(Method::new(
                                        mt,
                                        "TestFn".to_owned(),
                                        global::attr!(
                                            method Public {Static}
                                            /* 0 */ g_core_type!(System_USize), // Pointer to function

                                            /* 1 */ g_core_type!(System_UInt64),
                                            /* 2 */ g_core_type!(System_UInt32),
                                            /* 3 */ g_core_type!(System_UInt8),
                                            /* 4 */ g_core_type!(System_String),

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

                                            /* 16 */ g_core_type!(System_UInt64), // RET
                                        ),
                                        vec![],
                                        g_core_type!(System_UInt64),
                                        CallConvention::PlatformDefault,
                                        None,
                                        vec![
                                            // Load function pointer
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(0),
                                                val: test as *const u8 as usize as u64,
                                            },
                                            /* #region Arguments */
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(1),
                                                val: 0x1ff1,
                                            },
                                            Instruction::Load_u32 {
                                                register_addr: RegisterAddr::new(2),
                                                val: 10,
                                            },
                                            Instruction::Load_u8 {
                                                register_addr: RegisterAddr::new(3),
                                                val: 15,
                                            },
                                            Instruction::Load_String {
                                                register_addr: RegisterAddr::new(4),
                                                val: "aaa".to_owned(),
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
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method: System_NonPurusCallType_StaticMethodId::CreateU64.into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(7),
                                            },
                                            // Encoding
                                            Instruction::Load_u8 {
                                                register_addr: RegisterAddr::new(8),
                                                val: StringEncoding::C_Utf16.into(),
                                            },
                                            // Object strategy
                                            Instruction::Load_u8 {
                                                register_addr: RegisterAddr::new(9),
                                                val: ObjectStrategy::PointToData.into(),
                                            },
                                            // New by ref argument array
                                            Instruction::NewArray {
                                                element_type: CoreTypeId::System_USize
                                                    .static_type_ref()
                                                    .into(),
                                                len: 1,
                                                register_addr: RegisterAddr::new(10),
                                            },
                                            // New argument array
                                            Instruction::NewArray {
                                                element_type: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                len: 4,
                                                register_addr: RegisterAddr::new(11),
                                            },
                                            // Set Index
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 0,
                                            },
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(13),
                                                val: 1,
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(10),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(13),
                                                ],
                                                ret_at: RegisterAddr::new(15),
                                            },
                                            // Arg0
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreateU64
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
                                                ret_at: RegisterAddr::new(15),
                                            },
                                            // Arg 1
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 1,
                                            },
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreateU32
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
                                                ret_at: RegisterAddr::new(15),
                                            },
                                            // Arg 2
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 2,
                                            },
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreateU8
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
                                                ret_at: RegisterAddr::new(15),
                                            },
                                            // Arg 3
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 3,
                                            },
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreateString
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
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
                                    )),
                                    Box::new(
                                        Method::default_sctor(
                                            Some(mt),
                                            global::attr!(method Public {Static}),
                                        ),
                                    ),
                                ]
                            })
                            .as_non_null_ptr()
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

    let assem = global_vm()
        .assembly_manager()
        .get_assembly_by_name("Test")
        .unwrap()
        .unwrap();

    let class = assem.get_class(0).unwrap().unwrap();
    let mt = unsafe { class.as_ref().method_table_ref() };
    let test_fn = mt.find_first_method_by_name("TestFn").unwrap();
    let result = unsafe {
        test_fn
            .as_ref()
            .typed_res_call::<u64>(&cpu, None, &[])
    };
    dbg!(result);

    Ok(())
}

#[test]
#[cfg(windows)]
// cSpell:disable
fn message_box() {
    windows::core::link!(
        "user32.dll" "system" fn MessageBoxW(
            hwnd : windows::Win32::Foundation::HWND,
            lptext : windows::core::PCWSTR,
            lpcaption : windows::core::PCWSTR,
            utype : windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
        ) -> windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT
    );
    let f_ptr = MessageBoxW as *const u8;
    let cpu_id = global_vm().add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

    let cfg = NonPurusCallConfiguration {
        call_convention: CallConvention::PlatformDefault,
        return_type: NonPurusCallType::I32,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf16,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![
            (false, NonPurusCallType::Pointer),
            (false, NonPurusCallType::String),
            (false, NonPurusCallType::String),
            (false, NonPurusCallType::U32),
        ],
    };
    let hwnd = windows::Win32::Foundation::HWND(std::ptr::null_mut());
    let lptext = ManagedReference::new_string(&cpu, "TEXT");
    let lpcaption = ManagedReference::new_string(&cpu, "CAPTION");
    let utype = windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;

    let (result, result_layout) = cpu.non_purus_call(
        &cfg,
        f_ptr,
        vec![
            (&raw const hwnd).cast_mut().cast(),
            (&raw const lptext).cast_mut().cast(),
            (&raw const lpcaption).cast_mut().cast(),
            (&raw const utype).cast_mut().cast(),
        ],
    );
    unsafe {
        std::alloc::Allocator::deallocate(&std::alloc::Global, result, result_layout);
    }
}
// cSpell:enable

#[test]
#[cfg(windows)]
// cSpell:disable
fn dynamic_message_box() -> global::Result<()> {
    windows::core::link!(
        "user32.dll" "system" fn MessageBoxW(
            hwnd : windows::Win32::Foundation::HWND,
            lptext : windows::core::PCWSTR,
            lpcaption : windows::core::PCWSTR,
            utype : windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
        ) -> windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT
    );

    global_vm()
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
                                vec![
                                    Box::new(Method::new(
                                        mt,
                                        "TestFn".to_owned(),
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
                                        ),
                                        vec![],
                                        g_core_type!(System_Int32),
                                        CallConvention::PlatformDefault,
                                        None,
                                        vec![
                                            // Load function pointer
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(0),
                                                val: MessageBoxW as *const u8 as usize as u64,
                                            },
                                            /* #region Arguments */
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(1),
                                                val: 0,
                                            },
                                            Instruction::Load_String {
                                                register_addr: RegisterAddr::new(2),
                                                val: "TEXT".to_owned(),
                                            },
                                            Instruction::Load_String {
                                                register_addr: RegisterAddr::new(3),
                                                val: "CAPTION".to_owned(),
                                            },
                                            Instruction::Load_u32 {
                                                register_addr: RegisterAddr::new(4),
                                                val: windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR.0,
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
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method: System_NonPurusCallType_StaticMethodId::CreateI32.into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(7),
                                            },
                                            // Encoding
                                            Instruction::Load_u8 {
                                                register_addr: RegisterAddr::new(8),
                                                val: StringEncoding::C_Utf16.into(),
                                            },
                                            // Object strategy
                                            Instruction::Load_u8 {
                                                register_addr: RegisterAddr::new(9),
                                                val: ObjectStrategy::PointToData.into(),
                                            },
                                            // New by ref argument array
                                            Instruction::NewArray {
                                                element_type: CoreTypeId::System_USize
                                                    .static_type_ref()
                                                    .into(),
                                                len: 0,
                                                register_addr: RegisterAddr::new(10),
                                            },
                                            // New argument array
                                            Instruction::NewArray {
                                                element_type: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                len: 4,
                                                register_addr: RegisterAddr::new(11),
                                            },
                                            // Arg0
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 0,
                                            },
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreatePointer
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
                                                ret_at: RegisterAddr::new(15),
                                            },
                                            // Arg 1
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 1,
                                            },
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreateString
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
                                                ret_at: RegisterAddr::new(15),
                                            },
                                            // Arg 2
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 2,
                                            },
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreateString
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
                                                ret_at: RegisterAddr::new(15),
                                            },
                                            // Arg 3
                                            Instruction::Load_u64 {
                                                register_addr: RegisterAddr::new(12),
                                                val: 3,
                                            },
                                            Instruction::StaticCall {
                                                ty: CoreTypeId::System_NonPurusCallType
                                                    .static_type_ref()
                                                    .into(),
                                                method:
                                                    System_NonPurusCallType_StaticMethodId::CreateU32
                                                        .into(),
                                                args: vec![],
                                                ret_at: RegisterAddr::new(14),
                                            },
                                            Instruction::InstanceCall {
                                                val: RegisterAddr::new(11),
                                                method: System_Array_1_MethodId::set_Index.into(),
                                                args: vec![
                                                    RegisterAddr::new(12),
                                                    RegisterAddr::new(14),
                                                ],
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
                                    )),
                                    Box::new(
                                        Method::default_sctor(
                                            Some(mt),
                                            global::attr!(method Public {Static}),
                                        ),
                                    ),
                                ]
                            })
                            .as_non_null_ptr()
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

    let assem = global_vm()
        .assembly_manager()
        .get_assembly_by_name("Test")
        .unwrap()
        .unwrap();

    let class = assem.get_class(0).unwrap().unwrap();
    let mt = unsafe { class.as_ref().method_table_ref() };
    let test_fn = mt.find_first_method_by_name("TestFn").unwrap();
    let result = unsafe {
        test_fn
            .as_ref()
            .typed_res_call::<i32>(&cpu, None, &[])
    };
    dbg!(result);

    Ok(())
}
// cSpell:enable