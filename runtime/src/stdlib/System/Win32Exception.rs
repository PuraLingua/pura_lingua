use crate::stdlib::System::{_define_class, common_new_method, default_sctor};
use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

#[cfg(windows)]
pub fn Constructor(cpu: &mut CPU, method: &Method<Class>, this: &mut ManagedReference<Class>) {
    Constructor_I32(cpu, method, this, unsafe {
        windows::Win32::Foundation::GetLastError().0 as i32
    });
}

#[cfg(windows)]
/// Copy from [windows-rs](https://github.com/microsoft/windows-rs)
pub fn format_hresult(mut code: i32) -> Vec<u16> {
    #[repr(transparent)]
    struct HEAP_FLAGS(pub u32);

    windows::core::link!(
        "kernel32.dll"
        "system" fn LoadLibraryExA(
            lpLibFilename : windows::core::PCSTR,
            hfile : windows::Win32::Foundation::HANDLE,
            dwflags : windows::Win32::System::LibraryLoader::LOAD_LIBRARY_FLAGS,
        ) -> windows::Win32::Foundation::HMODULE
    );

    windows::core::link!(
        "kernel32.dll"
        "system" fn FormatMessageW(
            dwFlags : windows::Win32::System::Diagnostics::Debug::FORMAT_MESSAGE_OPTIONS,
            lpSource : *const core::ffi::c_void,
            dwMessageId : u32,
            dwLanguageId : u32,
            lpBuffer : windows::core::PWSTR,
            nSize : u32,
            arguments : *const *const i8
        ) -> u32
    );

    windows::core::link!(
        "kernel32.dll"
        "system" fn GetProcessHeap() -> windows::Win32::Foundation::HANDLE
    );
    windows::core::link!(
        "kernel32.dll"
        "system" fn HeapFree(
            hHeap: windows::Win32::Foundation::HANDLE,
            dwflags : HEAP_FLAGS,
            lpMem : *const core::ffi::c_void
        ) -> windows_result::BOOL
    );

    struct HeapString(*mut u16);

    impl Drop for HeapString {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let res = HeapFree(GetProcessHeap(), HEAP_FLAGS(0), self.0 as _);
                    cfg_select! {
                        debug_assertions => { if !res.as_bool() { panic!("HeapFree failed") } }
                        _ => { let _ = res; }
                    }
                }
            }
        }
    }

    fn wide_trim_end(mut wide: &[u16]) -> &[u16] {
        while let Some(last) = wide.last() {
            match last {
                32 | 9..=13 => wide = &wide[..wide.len() - 1],
                _ => break,
            }
        }
        wide
    }

    let mut message = HeapString(core::ptr::null_mut());
    let mut module = windows::Win32::Foundation::HMODULE(core::ptr::null_mut());

    let flags = {
        use windows::Win32::System::Diagnostics::Debug::*;
        let mut flags = FORMAT_MESSAGE_ALLOCATE_BUFFER
            | FORMAT_MESSAGE_FROM_SYSTEM
            | FORMAT_MESSAGE_IGNORE_INSERTS;

        if code & 0x1000_0000 == 0x1000_0000 {
            code ^= 0x1000_0000;
            flags |= FORMAT_MESSAGE_FROM_HMODULE;

            module = unsafe {
                LoadLibraryExA(
                    windows::core::PCSTR::from_raw(c"ntdll.dll".as_ptr() as _),
                    windows::Win32::Foundation::HANDLE(core::ptr::null_mut()),
                    windows::Win32::System::LibraryLoader::LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
                )
            };
        }

        flags
    };

    unsafe {
        let size = FormatMessageW(
            flags,
            module.0,
            code as _,
            0,
            windows::core::PWSTR::from_raw(&mut message.0 as *mut _ as *mut _),
            0,
            core::ptr::null(),
        );

        if !message.0.is_null() && size > 0 {
            wide_trim_end(core::slice::from_raw_parts(message.0, size as usize)).to_vec()
        } else {
            Vec::new()
        }
    }
}

#[cfg(windows)]
pub fn Constructor_I32(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
    code: i32,
) {
    use stdlib_header::definitions::System_Win32Exception_FieldId;

    assert!(
        this.const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_Win32Exception_FieldId::Code as _,
                Default::default(),
                code,
            )
    );

    let message = ManagedReference::new_string_from_wide(cpu, format_hresult(code));

    super::Exception::Constructor_String(cpu, method, this, message);
}

#[cfg(not(windows))]
pub fn Constructor(_: &CPU, _: &Method<Class>, _: &mut ManagedReference<Class>) {
    panic!("Unsupported");
}

#[cfg(not(windows))]
pub fn Constructor_I32(_: &CPU, _: &Method<Class>, _: &mut ManagedReference<Class>, _: i32) {
    panic!("Unsupported");
}

_define_class!(
    fn load(assembly, mt, method_info)
    System_Win32Exception
#methods(TMethodId):
    Constructor => common_new_method!(mt TMethodId Constructor Constructor);
    Constructor_I32 => common_new_method!(mt TMethodId Constructor_I32 Constructor_I32);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
