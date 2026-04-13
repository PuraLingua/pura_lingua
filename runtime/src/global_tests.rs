use global::instruction::{
    IRegisterAddr, Instruction, Instruction_Load, Instruction_UntypedCalculate, LoadContent,
    RegisterAddr,
};

use crate::{
    test_utils::{g_core_type, try_invoke_instructions},
    type_system::class::Class,
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::{CpuID, global_vm},
};

#[test]
#[cfg(unix)]
fn gtest_utf8() -> global::Result<()> {
    use std::ffi::c_int;

    use global::non_purus_call_configuration::{NonPurusCallConfiguration, NonPurusCallType};

    use crate::{test_utils::LEAK_DETECTOR, virtual_machine::cpu::CPU};

    let mut cpu = CpuID::new_write_global();

    let cfg = NonPurusCallConfiguration {
        call_convention: global::attrs::CallConvention::CDecl,
        return_type: NonPurusCallType::C_Int,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf8,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![(false, NonPurusCallType::String)],
    };

    let s = ManagedReference::new_string(&mut cpu, "aaa");

    LEAK_DETECTOR.scope_with(
        |cpu: &CPU, cfg, s| {
            use crate::virtual_machine::cpu::NonPurusCallArg;

            let (res_ptr, res_layout) = cpu.non_purus_call(
                cfg,
                libc::puts as _,
                vec![NonPurusCallArg::new(&s, NonPurusCallType::String)],
            );

            if unsafe { res_ptr.cast::<c_int>().read() } == libc::EOF {
                panic!("Failed to call puts");
            }

            unsafe {
                std::alloc::Allocator::deallocate(&std::alloc::Global, res_ptr, res_layout);
            }
        },
        (&*cpu, &cfg, s),
    );

    Ok(())
}

#[test]
#[cfg(windows)]
fn gtest_utf8() -> global::Result<()> {
    use std::os::raw::c_void;

    use global::non_purus_call_configuration::{NonPurusCallConfiguration, NonPurusCallType};

    use crate::{test_utils::LEAK_DETECTOR, virtual_machine::cpu::CPU};

    windows::core::link!("kernel32.dll" "system" fn WriteConsoleA(
        hConsoleOutput: windows::Win32::Foundation::HANDLE,
        lpBuffer : windows::core::PCSTR,
        nNumberOfCharsToWrite : u32,
        lpNumberOfCharsWritten : *mut u32,
        lpReserved : *const core::ffi::c_void
    ) -> windows::core::BOOL);

    let mut cpu = CpuID::new_write_global();

    let cfg = NonPurusCallConfiguration {
        call_convention: global::attrs::CallConvention::PlatformDefault,
        return_type: NonPurusCallType::I32,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf8,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![
            (false, NonPurusCallType::Pointer),
            (false, NonPurusCallType::String),
            (false, NonPurusCallType::U32),
            (true, NonPurusCallType::U32),
            (false, NonPurusCallType::Pointer),
        ],
    };

    let stdout_handle = unsafe {
        windows::Win32::System::Console::GetStdHandle(
            windows::Win32::System::Console::STD_OUTPUT_HANDLE,
        )?
    };
    let s = ManagedReference::new_string(&mut cpu, "aaa\n");
    let s_len = 4u32;
    let mut chars_written = 0;
    let reserved = std::ptr::null::<c_void>();

    LEAK_DETECTOR.scope_with(
        |cpu: &CPU, cfg, stdout_handle, s, s_len, chars_written, reserved| {
            use crate::virtual_machine::cpu::NonPurusCallArg;

            let (res_ptr, res_layout) = cpu.non_purus_call(
                cfg,
                WriteConsoleA as _,
                vec![
                    NonPurusCallArg::new(&stdout_handle, NonPurusCallType::Pointer),
                    NonPurusCallArg::new(&s, NonPurusCallType::String),
                    NonPurusCallArg::new(&s_len, NonPurusCallType::C_UInt),
                    NonPurusCallArg::new(chars_written, NonPurusCallType::C_Int),
                    NonPurusCallArg::new(&reserved, NonPurusCallType::Pointer),
                ],
            );

            if let Err(e) = unsafe { res_ptr.cast::<windows::core::BOOL>().read() }.ok() {
                panic!("Failed to call WriteConsoleA: {e}");
            }

            unsafe {
                std::alloc::Allocator::deallocate(&std::alloc::Global, res_ptr, res_layout);
            }
        },
        (
            &*cpu,
            &cfg,
            stdout_handle,
            s,
            s_len,
            &mut chars_written,
            reserved,
        ),
    );

    Ok(())
}

#[test]
fn gtest_test_fn() -> global::Result<()> {
    let vm = global_vm();

    let b_assembly = binary::assembly::AssemblyBuilder::from_path("../TestData/Test.plb")?;

    vm.assembly_manager()
        .load_binaries(&[binary::assembly::Assembly::from_builder(&b_assembly)])?;

    let mut cpu = CpuID::new_write_global();

    let assembly = vm
        .assembly_manager()
        .get_assembly_by_name("Test")
        .unwrap()
        .unwrap();

    let test_class = assembly.get_class(0).unwrap().unwrap();
    let test_fn = unsafe { test_class.as_ref() }
        .method_table_ref()
        .find_first_method_by_name("TestFn")
        .unwrap();

    let result =
        unsafe { test_fn.as_ref() }.typed_res_call::<ManagedReference<Class>>(&mut cpu, None, &[]);

    let res = result
        .access::<StringAccessor>()
        .unwrap()
        .get_str()
        .unwrap();
    println!("Result gotten: `{}`", res.display());

    Ok(())
}

#[test]
#[cfg(windows)]
fn gtest_test_msgbox() -> global::Result<()> {
    let vm = global_vm();

    let b_assembly = binary::assembly::AssemblyBuilder::from_path("../TestData/MsgboxTest.plb")?;

    vm.assembly_manager()
        .load_binaries(&[binary::assembly::Assembly::from_builder(&b_assembly)])?;

    let mut cpu = CpuID::new_write_global();

    let assembly = vm
        .assembly_manager()
        .get_assembly_by_name("MsgboxTest")
        .unwrap()
        .unwrap();

    let test_class = assembly.get_class(0).unwrap().unwrap();
    let test_fn = unsafe { test_class.as_ref() }
        .method_table_ref()
        .find_first_method_by_name("TestFn")
        .unwrap();

    let result = unsafe { test_fn.as_ref() }
        .typed_res_call::<windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT>(
        &mut cpu,
        None,
        &[],
    );

    println!(
        "You clicked {}",
        if result == windows::Win32::UI::WindowsAndMessaging::IDOK {
            "OK"
        } else {
            "<NOTHING>"
        }
    );

    Ok(())
}

#[test]
#[cfg(windows)]
fn gtest_simple_console() -> global::Result<()> {
    let vm = global_vm();

    let b_assembly =
        binary::assembly::AssemblyBuilder::from_path("../TestData/SimpleIR.SimpleConsole.plb")?;

    vm.assembly_manager()
        .load_binaries(&[binary::assembly::Assembly::from_builder(&b_assembly)])?;

    let mut cpu = CpuID::new_write_global();

    let assembly = vm
        .assembly_manager()
        .get_assembly_by_name("SimpleIR::SimpleConsole")
        .unwrap()
        .unwrap();

    let console_class = assembly.get_class(0).unwrap().unwrap();

    let to_write = ManagedReference::new_string(&mut cpu, "aaa\n");
    let write_stdout = unsafe {
        console_class
            .as_ref()
            .method_table_ref()
            .find_first_method_by_name("WriteStdout")
            .unwrap()
    };

    unsafe {
        write_stdout.as_ref().typed_res_call::<()>(
            &mut cpu,
            None,
            &[(&raw const to_write).cast_mut().cast()],
        );
    }

    Ok(())
}

#[test]
#[ignore = "Not yet ready"]
#[cfg(windows)]
fn gtest_middle_ir_simple_console() -> global::Result<()> {
    let vm = global_vm();

    let b_assembly =
        binary::assembly::AssemblyBuilder::from_path("../TestData/MiddleIR.SimpleConsole.plb")?;

    vm.assembly_manager()
        .load_binaries(&[binary::assembly::Assembly::from_builder(&b_assembly)])?;

    let mut cpu = CpuID::new_write_global();

    let assembly = vm
        .assembly_manager()
        .get_assembly_by_name("MiddleIR::SimpleConsole")
        .unwrap()
        .unwrap();

    let console_class = assembly.get_class(0).unwrap().unwrap();

    let to_write = ManagedReference::new_string(&mut cpu, "aaa\n");
    let write_stdout = unsafe {
        console_class
            .as_ref()
            .method_table_ref()
            .find_first_method_by_name("WriteStdout")
            .unwrap()
    };

    unsafe {
        write_stdout.as_ref().typed_res_call::<()>(
            &mut cpu,
            None,
            &[(&raw const to_write).cast_mut().cast()],
        );
    }

    Ok(())
}

#[test]
fn calculating() {
    let (res_ptr, res_layout) = try_invoke_instructions(
        vec![
            g_core_type!(System_UInt64),
            g_core_type!(System_UInt64),
            g_core_type!(System_UInt64),
            g_core_type!(System_UInt64),
            g_core_type!(System_UInt64),
        ],
        g_core_type!(System_UInt64),
        vec![
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(0),
                content: LoadContent::U64(0),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(1),
                content: LoadContent::U64(1),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(2),
                content: LoadContent::U64(2),
            }),
            Instruction::Load(Instruction_Load {
                addr: RegisterAddr::new(3),
                content: LoadContent::U64(3),
            }),
            Instruction::Calculate(
                Instruction_UntypedCalculate::<_, u64>::Add {
                    lhs: RegisterAddr::new(0),
                    rhs: RegisterAddr::new(4),
                    target: RegisterAddr::new(4),
                }
                .into(),
            ),
            Instruction::Calculate(
                Instruction_UntypedCalculate::<_, u64>::Add {
                    lhs: RegisterAddr::new(1),
                    rhs: RegisterAddr::new(4),
                    target: RegisterAddr::new(4),
                }
                .into(),
            ),
            Instruction::Calculate(
                Instruction_UntypedCalculate::<_, u64>::Add {
                    lhs: RegisterAddr::new(2),
                    rhs: RegisterAddr::new(4),
                    target: RegisterAddr::new(4),
                }
                .into(),
            ),
            Instruction::Calculate(
                Instruction_UntypedCalculate::<_, u64>::Add {
                    lhs: RegisterAddr::new(3),
                    rhs: RegisterAddr::new(4),
                    target: RegisterAddr::new(4),
                }
                .into(),
            ),
            Instruction::ReturnVal {
                register_addr: RegisterAddr::new(4),
            },
        ],
    );
    unsafe {
        assert_eq!(res_ptr.cast::<u64>().read(), 0 + 1 + 2 + 3);
        std::alloc::Allocator::deallocate(&std::alloc::Global, res_ptr, res_layout);
    }
}
