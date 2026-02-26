use crate::{
    type_system::class::Class,
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::{CpuID, global_vm},
};

#[test]
#[cfg(unix)]
fn gtest_utf8() -> global::Result<()> {
    use std::ffi::c_int;

    use global::non_purus_call_configuration::{NonPurusCallConfiguration, NonPurusCallType};

    let cpu = CpuID::new_global();

    let cfg = NonPurusCallConfiguration {
        call_convention: global::attrs::CallConvention::CDecl,
        return_type: NonPurusCallType::C_Int,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf8,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![(false, NonPurusCallType::String)],
    };

    let s = ManagedReference::new_string(&cpu, "aaa");
    let (res_ptr, _res_layout) = cpu.non_purus_call(
        &cfg,
        libc::puts as _,
        vec![(&raw const s).cast_mut().cast()],
    );

    unsafe {
        libc::puts(c"bbb".as_ptr());
    }
    if unsafe { res_ptr.cast::<c_int>().read() } == libc::EOF {
        panic!("Failed to call puts");
    }

    Ok(())
}

#[test]
#[cfg(windows)]
fn gtest_utf8() -> global::Result<()> {
    use std::{ffi::c_int, os::raw::c_void};

    use global::non_purus_call_configuration::{NonPurusCallConfiguration, NonPurusCallType};

    windows::core::link!("kernel32.dll" "system" fn WriteConsoleA(
        hConsoleOutput: windows::Win32::Foundation::HANDLE,
        lpBuffer : windows::core::PCSTR,
        nNumberOfCharsToWrite : u32,
        lpNumberOfCharsWritten : *mut u32,
        lpReserved : *const core::ffi::c_void
    ) -> windows::core::BOOL);

    let cpu = CpuID::new_global();

    let cfg = NonPurusCallConfiguration {
        call_convention: global::attrs::CallConvention::CDecl,
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
    let s = ManagedReference::new_string(&cpu, "aaa");
    let s_len = 3u32;
    let mut chars_written = 0;
    let reserved = std::ptr::null::<c_void>();
    let (res_ptr, _res_layout) = cpu.non_purus_call(
        &cfg,
        WriteConsoleA as _,
        vec![
            (&raw const stdout_handle).cast_mut().cast(),
            (&raw const s).cast_mut().cast(),
            (&raw const s_len).cast_mut().cast(),
            (&raw mut chars_written).cast(),
            (&raw const reserved).cast_mut().cast(),
        ],
    );

    if let Err(e) = unsafe { res_ptr.cast::<windows::core::BOOL>().read() }.ok() {
        panic!("Failed to call WriteConsoleA: {e}");
    }

    Ok(())
}

#[test]
fn gtest_test_fn() -> global::Result<()> {
    let vm = global_vm();

    vm.assembly_manager()
        .load_binaries(&[binary::assembly::Assembly::from_path(
            "../TestData/Test.plb",
        )?])?;

    let cpu_id = vm.add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

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
        unsafe { test_fn.as_ref() }.typed_res_call::<ManagedReference<Class>>(&cpu, None, &[]);

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

    vm.assembly_manager()
        .load_binaries(&[binary::assembly::Assembly::from_path(
            "../TestData/MsgboxTest.plb",
        )?])?;

    let cpu_id = vm.add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

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
        &cpu,
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

    vm.assembly_manager()
        .load_binaries(&[binary::assembly::Assembly::from_path(
            "../TestData/System_SimpleConsole.plb",
        )?])?;

    let cpu_id = vm.add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

    let assembly = vm
        .assembly_manager()
        .get_assembly_by_name("System_SimpleConsole")
        .unwrap()
        .unwrap();

    let console_class = assembly.get_class(0).unwrap().unwrap();

    let to_write = ManagedReference::new_string(&cpu, "aaa\n");
    let write_stdout = unsafe {
        console_class
            .as_ref()
            .method_table_ref()
            .find_first_method_by_name("WriteStdout")
            .unwrap()
    };

    unsafe {
        write_stdout.as_ref().typed_res_call::<()>(
            &cpu,
            None,
            &[(&raw const to_write).cast_mut().cast()],
        );
    }

    Ok(())
}
