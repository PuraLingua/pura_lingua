use std::num::NonZero;

use crate::{
    type_system::class::Class,
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::global_vm,
};

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

#[cfg(windows)]
fn to_multi_byte(name_wide: &widestring::U16CStr) -> Option<(Vec<u8>, Option<NonZero<i32>>)> {
    windows::core::link!(
        "kernel32.dll" "system" fn WideCharToMultiByte(
            CodePage: u32,
            dwFlags: u32,
            lpWideCharStr: windows::core::PCWSTR,
            cchWideChar: i32,
            lpMultiByteStr: windows::core::PSTR,
            cbMultiByte: i32,
            lpDefaultChar: windows::core::PCSTR,
            lpUsedDefaultChar: *mut windows::core::BOOL
        ) -> i32
    );

    let len = unsafe {
        WideCharToMultiByte(
            windows::Win32::Globalization::CP_ACP,
            0,
            windows::core::PCWSTR::from_raw(name_wide.as_ptr()),
            -1,
            windows::core::PSTR::null(),
            0,
            windows::core::PCSTR::null(),
            std::ptr::null_mut(),
        ) as usize
    };
    let mut name_out = vec![0u8; len];
    let used_len = unsafe {
        WideCharToMultiByte(
            windows::Win32::Globalization::CP_ACP,
            0,
            windows::core::PCWSTR::from_raw(name_wide.as_ptr()),
            -1,
            windows::core::PSTR::from_raw(name_out.as_mut_ptr()),
            len as _,
            windows::core::PCSTR::null(),
            std::ptr::null_mut(),
        )
    };

    let used_len_non_zero = NonZero::new(used_len);

    Some((name_out, used_len_non_zero))
}

#[cfg(windows)]
static KERNEL32_NAME: &widestring::U16CStr = widestring::u16cstr!("Kernel32.dll");
#[cfg(windows)]
static GET_STD_HANDLE_NAME: &widestring::U16CStr = widestring::u16cstr!("GetStdHandle");
#[cfg(windows)]
static STD_OUTPUT_HANDLE: u32 = -11i32 as u32;

#[cfg(windows)]
type TGetStdHandle = extern "system" fn(u32) -> *const u8;

#[test]
#[cfg(windows)]
fn write_console_test() -> global::Result<()> {
    use std::alloc::Layout;

    use global::non_purus_call_configuration::{NonPurusCallConfiguration, NonPurusCallType};
    use windows::core::PCWSTR;

    let addr_kernel32 = unsafe {
        windows::Win32::System::LibraryLoader::LoadLibraryW(PCWSTR::from_raw(
            KERNEL32_NAME.as_ptr(),
        ))?
    };

    let cfg_get_std_handle = NonPurusCallConfiguration {
        call_convention: global::attrs::CallConvention::PlatformDefault,
        return_type: NonPurusCallType::Pointer,
        encoding: global::non_purus_call_configuration::StringEncoding::C_Utf16,
        object_strategy: global::non_purus_call_configuration::ObjectStrategy::PointToData,
        arguments: vec![(false, NonPurusCallType::U32)],
    };
    let (get_std_handle_name_multi_byte, Some(_)) = to_multi_byte(GET_STD_HANDLE_NAME).unwrap()
    else {
        panic!("WideCharToMultiByte failed");
    };
    let addr_get_std_handle = unsafe {
        windows::Win32::System::LibraryLoader::GetProcAddress(
            addr_kernel32,
            windows::core::PCSTR::from_raw(get_std_handle_name_multi_byte.as_ptr()),
        )
        .unwrap() as *const u8
    };

    let vm = global_vm();

    let cpu_id = vm.add_cpu();
    let cpu = cpu_id.as_global_cpu().unwrap();

    let (stdout_handle_ptr, stdout_handle_layout) = cpu.non_purus_call(
        &cfg_get_std_handle,
        addr_get_std_handle,
        vec![(&raw const STD_OUTPUT_HANDLE).cast_mut().cast()],
    );
    assert_eq!(
        stdout_handle_layout,
        Layout::new::<windows::Win32::Foundation::HANDLE>()
    );
    let stdout_handle = unsafe {
        stdout_handle_ptr
            .cast::<windows::Win32::Foundation::HANDLE>()
            .read()
    };
    dbg!(stdout_handle);

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
