use std::alloc::{Allocator, Layout};

use global::{
    attrs::CallConvention,
    instruction::{
        IRegisterAddr, Instruction, Instruction_Calculate, Instruction_Call, Instruction_Load,
        Instruction_New, Instruction_UntypedCalculate, LoadContent, RegisterAddr,
    },
    non_purus_call_configuration::{
        NonPurusCallConfiguration, NonPurusCallType, ObjectStrategy, StringEncoding,
    },
};
use stdlib_header::definitions::{
    System_Array_1_MethodId, System_NonPurusCallConfiguration_MethodId,
    System_NonPurusCallType_StaticMethodId,
};
use widestring::U16CStr;

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdConstExt, CoreTypeIdExt as _},
    test_utils::{LEAK_DETECTOR, g_core_type, try_invoke_instructions},
    type_system::{
        assembly::Assembly,
        class::Class,
        method::Method,
        method_table::MethodTable,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    virtual_machine::{CpuID, EnsureGlobalVirtualMachineInitialized, global_vm},
};

use super::*;

#[test]
fn test_call_stack() {
    EnsureGlobalVirtualMachineInitialized();

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
                        vec![],
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

    let mut cpu = CpuID::new_write_global();
    cpu.prepare_call_stack_for_method(unsafe { method.as_ref() });

    let call_frame = cpu.current_common_call_frame().unwrap();
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
    let mut cpu = cpu_id.as_global_write_cpu().unwrap();

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
    let mut d = ManagedReference::new_string(&mut cpu, "aaa");
    let (result, result_layout) = cpu.non_purus_call(
        &cfg,
        f_ptr,
        vec![
            NonPurusCallArg::new(&a, NonPurusCallType::U64),
            NonPurusCallArg::new(&b, NonPurusCallType::U32),
            NonPurusCallArg::new(&c, NonPurusCallType::U8),
            NonPurusCallArg::new(&d, NonPurusCallType::String),
        ],
    );
    unsafe {
        dbg!(result.cast::<u64>().as_ref());
    }
    d.destroy(&mut cpu);

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
    let mut cpu = CpuID::new_write_global();

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
    let mut d = ManagedReference::new_string(&mut cpu, "aaa");
    let (result, result_layout) = cpu.non_purus_call(
        &cfg,
        f_ptr,
        vec![
            NonPurusCallArg::new(&a, NonPurusCallType::U64),
            NonPurusCallArg::new(&b, NonPurusCallType::U32),
            NonPurusCallArg::new(&c, NonPurusCallType::U8),
            NonPurusCallArg::new(&d, NonPurusCallType::String),
        ],
    );
    unsafe {
        dbg!(result.cast::<u64>().as_ref());
    }
    d.destroy(&mut cpu);

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

    let (result_ptr, result_layout) = try_invoke_instructions(
        vec![
            /* 0 */ g_core_type!(System_USize), // Pointer to function
            /* 1 */ g_core_type!(System_UInt64), // a
            /* 2 */ g_core_type!(System_UInt32), // b
            /* 3 */ g_core_type!(System_UInt8), // c
            /* 4 */ g_core_type!(System_String), // d
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
        ],
        g_core_type!(System_UInt64),
        vec![
            // Load function pointer
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(0),
                content: LoadContent::U64(test as *const u8 as usize as u64),
            }),
            /* #region Arguments */
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(1),
                content: LoadContent::U64(0x1ff1),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(2),
                content: LoadContent::U32(10),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(3),
                content: LoadContent::U8(15),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(4),
                content: LoadContent::String("aaa".to_owned()),
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
                method: System_NonPurusCallType_StaticMethodId::CreateU64.into(),
                args: vec![],
                ret_at: RegisterAddr::new(7),
            }),
            // Encoding
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(8),
                content: LoadContent::U8(StringEncoding::C_Utf16.into()),
            }),
            // Object strategy
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(9),
                content: LoadContent::U8(ObjectStrategy::PointToData.into()),
            }),
            // New by ref argument array
            Instruction::New(Instruction_New::NewArray {
                element_type: CoreTypeId::System_USize.static_type_ref().into(),
                len: 1,
                output: RegisterAddr::new(10),
            }),
            // New argument array
            Instruction::New(Instruction_New::NewArray {
                element_type: CoreTypeId::System_USize.static_type_ref().into(),
                len: 4,
                output: RegisterAddr::new(11),
            }),
            // Set Index
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(12),
                content: LoadContent::U64(0),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(13),
                content: LoadContent::U64(1),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(10),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(13)],
                ret_at: RegisterAddr::new(15),
            }),
            // Arg0
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateU64.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            // Arg 1
            Instruction::Calculate(Instruction_Calculate::U64(
                Instruction_UntypedCalculate::AddOne {
                    target: RegisterAddr::new(12),
                },
            )),
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateU32.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            // Arg 2
            Instruction::Calculate(Instruction_Calculate::U64(
                Instruction_UntypedCalculate::AddOne {
                    target: RegisterAddr::new(12),
                },
            )),
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateU8.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            // Arg 3
            Instruction::Calculate(Instruction_Calculate::U64(
                Instruction_UntypedCalculate::AddOne {
                    target: RegisterAddr::new(12),
                },
            )),
            Instruction::Call(Instruction_Call::StaticCall {
                ty: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
                method: System_NonPurusCallType_StaticMethodId::CreateString.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            /* #endregion */
            // Construct
            Instruction::New(Instruction_New::NewObject {
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
    );

    let result = unsafe {
        let data = result_ptr.cast::<u64>().read();
        std::alloc::Allocator::deallocate(&std::alloc::Global, result_ptr, result_layout);
        data
    };
    dbg!(result);

    Ok(())
}

#[test]
fn non_purus_call_va_arg() {
    unsafe extern "C" {
        safe fn wprintf(format: *const u16, ...) -> std::ffi::c_int;
    }

    let f_ptr = wprintf as *const u8;
    let mut cpu = CpuID::new_write_global();

    let cfg = NonPurusCallConfiguration {
        call_convention: CallConvention::CDeclWithVararg,
        return_type: NonPurusCallType::I32,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf16,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![(false, NonPurusCallType::String)],
    };

    let formatting = ManagedReference::new_string(&mut cpu, "%ls: %d\n");
    let info = ManagedReference::new_string(&mut cpu, "AAA");
    let i_data: std::ffi::c_int = 10;

    let (result, result_layout) = cpu.non_purus_call(
        &cfg,
        f_ptr,
        vec![
            NonPurusCallArg::new(&formatting, NonPurusCallType::String),
            NonPurusCallArg::new(&info, NonPurusCallType::String),
            NonPurusCallArg::new(&i_data, NonPurusCallType::C_Int),
        ],
    );
    unsafe {
        std::alloc::Allocator::deallocate(&std::alloc::Global, result, result_layout);
    }
}

#[test]
#[cfg(windows)]
// cSpell:disable
fn static_message_box() {
    windows::core::link!(
        "user32.dll" "system" fn MessageBoxW(
            hwnd : windows::Win32::Foundation::HWND,
            lptext : windows::core::PCWSTR,
            lpcaption : windows::core::PCWSTR,
            utype : windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
        ) -> windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT
    );
    let f_ptr = MessageBoxW as *const u8;
    let mut cpu = CpuID::new_write_global();

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
    let lptext = ManagedReference::new_string(&mut cpu, "TEXT");
    let lpcaption = ManagedReference::new_string(&mut cpu, "CAPTION");
    let utype = windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;

    let (result, result_layout) = cpu.non_purus_call(
        &cfg,
        f_ptr,
        vec![
            NonPurusCallArg::new(&hwnd, NonPurusCallType::Pointer),
            NonPurusCallArg::new(&lptext, NonPurusCallType::String),
            NonPurusCallArg::new(&lpcaption, NonPurusCallType::String),
            NonPurusCallArg::new(&utype, NonPurusCallType::U32),
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
    use crate::test_utils::try_invoke_instructions;

    windows::core::link!(
        "user32.dll" "system" fn MessageBoxW(
            hwnd : windows::Win32::Foundation::HWND,
            lpText : windows::core::PCWSTR,
            lpCaption : windows::core::PCWSTR,
            uType : windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
        ) -> windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT
    );

    let (result_ptr, result_layout) = try_invoke_instructions(
        vec![
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
        ],
        g_core_type!(System_Int32),
        vec![
            // Load function pointer
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(0),
                content: LoadContent::U64(MessageBoxW as *const u8 as usize as u64),
            }),
            /* #region Arguments */
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(1),
                content: LoadContent::U64(0),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(2),
                content: LoadContent::String("TEXT".to_owned()),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(3),
                content: LoadContent::String("CAPTION".to_owned()),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(4),
                content: LoadContent::U32(windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR.0),
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
                method: System_NonPurusCallType_StaticMethodId::CreateI32.into(),
                args: vec![],
                ret_at: RegisterAddr::new(7),
            }),
            // Encoding
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(8),
                content: LoadContent::U8(StringEncoding::C_Utf16.into()),
            }),
            // Object strategy
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(9),
                content: LoadContent::U8(ObjectStrategy::PointToData.into()),
            }),
            // New by ref argument array
            Instruction::New(Instruction_New::NewArray {
                element_type: CoreTypeId::System_USize.static_type_ref().into(),
                len: 0,
                output: RegisterAddr::new(10),
            }),
            // New argument array
            Instruction::New(Instruction_New::NewArray {
                element_type: CoreTypeId::System_NonPurusCallType.static_type_ref().into(),
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
                method: System_NonPurusCallType_StaticMethodId::CreatePointer.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
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
                method: System_NonPurusCallType_StaticMethodId::CreateString.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
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
                method: System_NonPurusCallType_StaticMethodId::CreateString.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
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
                method: System_NonPurusCallType_StaticMethodId::CreateU32.into(),
                args: vec![],
                ret_at: RegisterAddr::new(14),
            }),
            Instruction::Call(Instruction_Call::InstanceCall {
                val: RegisterAddr::new(11),
                method: System_Array_1_MethodId::set_Index.into(),
                args: vec![RegisterAddr::new(12), RegisterAddr::new(14)],
                ret_at: RegisterAddr::new(15),
            }),
            /* #endregion */
            // Construct
            Instruction::New(Instruction_New::NewObject {
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
    );

    let result = unsafe {
        let data = result_ptr.cast::<i32>().read();
        std::alloc::Allocator::deallocate(&std::alloc::Global, result_ptr, result_layout);
        data
    };
    dbg!(result);

    Ok(())
}
// cSpell:enable
