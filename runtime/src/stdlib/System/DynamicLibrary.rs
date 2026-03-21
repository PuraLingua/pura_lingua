use std::ffi::c_void;
use std::ptr::NonNull;

use global::t_println;
use stdlib_header::definitions::System_DynamicLibrary_FieldId;

use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

#[cfg(test)]
mod tests;

pub extern "system" fn Constructor_String(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    file: ManagedReference<Class>,
) {
    t_println!(
        "Loading lib: {}",
        file.access::<StringAccessor>()
            .unwrap()
            .get_str()
            .unwrap()
            .display()
    );
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
    println!("finish loading successfully");
}

pub extern "system" fn GetSymbol(
    cpu: &mut CPU,
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
    cpu: &mut CPU,
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
fn LoadLibrary(cpu: &mut CPU, file: ManagedReference<Class>) -> Option<NonNull<c_void>> {
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
            assert!(cpu.throw_helper_mut().win32(e.code().0));
            None
        }
    }
}

#[cfg(windows)]
fn GetSymbolImpl(
    cpu: &mut CPU,
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
        let (name_out, used_len) = name
            .access::<StringAccessor>()
            .unwrap()
            .to_multi_byte()
            .unwrap();

        if used_len.is_none() {
            assert!(cpu.throw_helper_mut().current_win32());
            return None;
        }
        match windows::Win32::System::LibraryLoader::GetProcAddress(
            hModule,
            windows::core::PCSTR::from_raw(name_out.as_ptr()),
        ) {
            Some(x) => Some(NonNull::new_unchecked(x as _)),
            None => {
                assert!(cpu.throw_helper_mut().current_win32());
                None
            }
        }
    }
}

#[cfg(windows)]
#[must_use]
fn FreeLibrary(cpu: &mut CPU, handle: *mut c_void) -> bool {
    let hLibModule = windows::Win32::Foundation::HMODULE(handle);
    match unsafe { windows::Win32::Foundation::FreeLibrary(hLibModule) } {
        Ok(_) => true,
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper_mut().win32(e.code().0));
            false
        }
    }
}

#[cfg(unix)]
fn LoadLibrary(cpu: &mut CPU, file: ManagedReference<Class>) -> Option<NonNull<c_void>> {
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
            assert!(cpu.throw_helper_mut().current_dlerror());
            None
        }
    }
}

#[cfg(unix)]
fn GetSymbolImpl(
    cpu: &mut CPU,
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
            assert!(cpu.throw_helper_mut().current_dlerror());
            None
        }
    }
}

#[cfg(unix)]
#[must_use]
fn FreeLibrary(cpu: &mut CPU, handle: *mut c_void) -> bool {
    if handle.is_null() {
        return true;
    }
    unsafe {
        if libc::dlclose(handle) != 0 {
            assert!(cpu.throw_helper_mut().current_dlerror());
            false
        } else {
            true
        }
    }
}
// cSpell:enable

_define_class!(
    fn load(assembly, mt, method_info)
    System_DynamicLibrary
#methods(TMethodId):
    Destructor => common_new_method!(mt TMethodId Destructor Destructor);
    Constructor_String => common_new_method!(mt TMethodId Constructor_String Constructor_String);
    GetSymbol => common_new_method!(mt TMethodId GetSymbol GetSymbol);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
