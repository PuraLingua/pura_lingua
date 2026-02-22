use std::ffi::c_void;
use std::ptr::NonNull;

use crate::{
    stdlib::System_DynamicLibrary_FieldId,
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

#[cfg(test)]
mod tests;

pub extern "system" fn Constructor_String(
    cpu: &CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    file: ManagedReference<Class>,
) {
    let handle_out = this
        .const_access_mut::<FieldAccessor<Class>>()
        .typed_field_mut::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    let Some(handle) = LoadLibrary(cpu, file) else {
        return;
    };
    *handle_out = handle.as_ptr();
}

pub extern "system" fn GetSymbol(
    cpu: &CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    let handle = this
        .const_access_mut::<FieldAccessor<Class>>()
        .read_typed_field::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    GetSymbolImpl(cpu, handle, name)
}

pub extern "system" fn Destructor(
    cpu: &CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
) {
    let handle = this
        .const_access_mut::<FieldAccessor<Class>>()
        .read_typed_field::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    if !FreeLibrary(cpu, handle) {
        return;
    }
    ClearHandlePtr(this);
}

fn ClearHandlePtr(this: &mut ManagedReference<Class>) {
    assert!(
        this.const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field::<*mut c_void>(
                System_DynamicLibrary_FieldId::Handle as _,
                Default::default(),
                std::ptr::null_mut()
            )
    );
}

// cSpell:disable
#[cfg(windows)]
fn LoadLibrary(cpu: &CPU, file: ManagedReference<Class>) -> Option<NonNull<c_void>> {
    match unsafe {
        windows::Win32::System::LibraryLoader::LoadLibraryW(windows::core::PCWSTR::from_raw(
            file.access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap()
                .as_ptr(),
        ))
    } {
        Ok(x) => Some(unsafe { NonNull::new_unchecked(x.0) }),
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper().win32(e.code().0));
            None
        }
    }
}

#[cfg(windows)]
fn GetSymbolImpl(
    cpu: &CPU,
    handle: *mut c_void,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    windows::core::link!(
        "kernel32.dll" "system" fn WideCharToMultiByte(
            codepage: u32,
            dwflags: u32,
            lpwidecharstr: windows::core::PCWSTR,
            cchwidechar: i32,
            lpmultibytestr: windows::core::PSTR,
            cbmultibyte: i32,
            lpdefaultchar: windows::core::PCSTR,
            lpuseddefaultchar: *mut windows::core::BOOL
        ) -> i32
    );

    let hModule = windows::Win32::Foundation::HMODULE(handle);

    unsafe {
        let name_wide = name.access::<StringAccessor>().unwrap().get_str().unwrap();
        let len = WideCharToMultiByte(
            windows::Win32::Globalization::CP_ACP,
            0,
            windows::core::PCWSTR::from_raw(name_wide.as_ptr()),
            -1,
            windows::core::PSTR::null(),
            0,
            windows::core::PCSTR::null(),
            std::ptr::null_mut(),
        ) as usize;
        let mut name_out = vec![0u8; len];
        let used_len = WideCharToMultiByte(
            windows::Win32::Globalization::CP_ACP,
            0,
            windows::core::PCWSTR::from_raw(name_wide.as_ptr()),
            -1,
            windows::core::PSTR::from_raw(name_out.as_mut_ptr()),
            len as _,
            windows::core::PCSTR::null(),
            std::ptr::null_mut(),
        );

        if used_len == 0 {
            assert!(cpu.throw_helper().current_win32());
        }
        match windows::Win32::System::LibraryLoader::GetProcAddress(
            hModule,
            windows::core::PCSTR::from_raw(name_out.as_ptr()),
        ) {
            Some(x) => Some(NonNull::new_unchecked(x as _)),
            None => {
                assert!(cpu.throw_helper().current_win32());
                None
            }
        }
    }
}

#[cfg(windows)]
#[must_use]
fn FreeLibrary(cpu: &CPU, handle: *mut c_void) -> bool {
    let hLibModule = windows::Win32::Foundation::HMODULE(handle);
    match unsafe { windows::Win32::Foundation::FreeLibrary(hLibModule) } {
        Ok(_) => true,
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper().win32(e.code().0));
            false
        }
    }
}

#[cfg(unix)]
fn LoadLibrary(cpu: &CPU, file: ManagedReference<Class>) -> Option<NonNull<c_void>> {
    let file_path = std::ffi::CString::new(
        file.access::<StringAccessor>()
            .unwrap()
            .to_string()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let handle = unsafe { libc::dlopen(file_path.as_ptr(), libc::RTLD_LAZY) };
    match NonNull::new(handle) {
        Some(x) => Some(x),
        None => {
            assert!(cpu.throw_helper().current_dlerror());
            None
        }
    }
}

#[cfg(unix)]
fn GetSymbolImpl(
    cpu: &CPU,
    handle: *mut c_void,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    let c_name = std::ffi::CString::new(
        name.access::<StringAccessor>()
            .unwrap()
            .to_string()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    match NonNull::new(unsafe { libc::dlsym(handle, c_name.as_ptr()) }) {
        Some(x) => Some(x),
        None => {
            assert!(cpu.throw_helper().current_dlerror());
            None
        }
    }
}

#[cfg(unix)]
#[must_use]
fn FreeLibrary(cpu: &CPU, handle: *mut c_void) -> bool {
    if handle.is_null() {
        return true;
    }
    unsafe {
        if libc::dlclose(handle) != 0 {
            assert!(cpu.throw_helper().current_dlerror());
            false
        } else {
            true
        }
    }
}
// cSpell:enable
