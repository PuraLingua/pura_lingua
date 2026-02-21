use std::ffi::c_void;

use crate::{
    stdlib::System_DynamicLibrary_FieldId,
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

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
    cfg_select! {
        windows => {
            let Some(handle) = LoadLibrary(cpu, file) else {
                return;
            };
            debug_assert!(!handle.is_invalid());
            *handle_out = handle.0;
        }
        unix => {
            let file_path = std::ffi::CString::new(file.access::<StringAccessor>().unwrap().to_string().unwrap());
            let handle = libc::dlopen(file_path.as_ptr(), libc::RTLD_LAZY);
        }
    }
}

// cSpell:disable
#[cfg(windows)]
fn LoadLibrary(
    cpu: &CPU,
    file: ManagedReference<Class>,
) -> Option<windows::Win32::Foundation::HMODULE> {
    match unsafe {
        windows::Win32::System::LibraryLoader::LoadLibraryW(windows::core::PCWSTR::from_raw(
            file.access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap()
                .as_ptr(),
        ))
    } {
        Ok(x) => Some(x),
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper().win32(e.code().0));
            None
        }
    }
}

#[cfg(unix)]
fn LoadLibrary(cpu: &CPU, file: ManagedReference<Class>) -> Option<NonNull<c_void>> {}

// cSpell:enable
