pub(in super::super) fn LoadLibraryImpl(
    cpu: &mut CPU,
    file: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    match LoadLibraryInner(file.access::<StringAccessor>().unwrap().get_str().unwrap()) {
        Ok(x) => Some(unsafe { NonNull::new_unchecked(x.0) }),
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper_mut().win32(e.code().0));
            None
        }
    }
}

pub(super) fn LoadLibraryInner(
    name: &U16CStr,
) -> windows::core::Result<windows::Win32::Foundation::HMODULE> {
    let name = match name {
        x if x == widestring::u16cstr!("$RUNTIME") => {
            return unsafe {
                windows::Win32::System::LibraryLoader::LoadLibraryW(windows::core::PCWSTR::null())
            };
        }

        _ => Cow::Borrowed(name),
    };

    unsafe { windows::Win32::System::LibraryLoader::LoadLibraryW(name.as_ptr()) }
}

pub(in super::super) fn GetSymbolImpl(
    cpu: &mut CPU,
    handle: *mut c_void,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
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

#[must_use]
pub(in super::super) fn FreeLibraryImpl(cpu: &mut CPU, handle: *mut c_void) -> bool {
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
