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
