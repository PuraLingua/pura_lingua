use global::attrs::CallConvention;

pub const fn get_abi_by_call_convention(call_convention: CallConvention) -> libffi::raw::ffi_abi {
    match call_convention {
        CallConvention::PlatformDefault => libffi::middle::ffi_abi_FFI_DEFAULT_ABI,
        CallConvention::CDecl => cfg_select! {
            target_arch = "x86" => { libffi::middle::ffi_abi_FFI_MS_CDECL }
            _ => { libffi::raw::ffi_abi_FFI_WIN64 }
        },
        CallConvention::CDeclWithVararg => cfg_select! {
            target_arch = "x86" => { libffi::middle::ffi_abi_FFI_MS_CDECL }
            _ => { libffi::raw::ffi_abi_FFI_WIN64 }
        },
        CallConvention::Stdcall => cfg_select! {
            target_arch = "x86" => { libffi::middle::ffi_abi_FFI_STDCALL }
            _ => { libffi::raw::ffi_abi_FFI_WIN64 }
        },
        CallConvention::Fastcall => cfg_select! {
            target_arch = "x86" => { libffi::middle::ffi_abi_FFI_FASTCALL }
            _ => { libffi::raw::ffi_abi_FFI_DEFAULT_ABI }
        },
        CallConvention::Win64 => cfg_select! {
            all(windows, target_arch = "x86_64") => { libffi::raw::ffi_abi_FFI_WIN64 }
            _ => { libffi::raw::ffi_abi_FFI_DEFAULT_ABI }
        },
        CallConvention::SystemV => cfg_select! {
            /* cSpell:disable-next-line */
            all(unix, target_arch = "x86_64") => { libffi::raw::ffi_abi_FFI_GNUW64 }
            _ => { libffi::raw::ffi_abi_FFI_DEFAULT_ABI }
        },
    }
}
