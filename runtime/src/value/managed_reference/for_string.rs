use std::{alloc::Layout, mem::offset_of, ptr::NonNull};

use widestring::U16CStr;

use crate::{
    stdlib::CoreTypeId,
    type_system::{class::Class, method_table::MethodTable, type_handle::NonGenericTypeHandleKind},
    value::{managed_reference::ManagedReferenceInner, object_header::ObjectHeader},
    virtual_machine::cpu::CPU,
};

use super::{IAccessor, ManagedReference};

impl ManagedReference<Class> {
    #[track_caller]
    pub fn new_string(cpu: &CPU, s: &str) -> Self {
        Self::new_string_from_wide(cpu, s.encode_utf16().collect())
    }

    #[track_caller]
    pub fn new_string_from_wide(cpu: &CPU, mut bytes: Vec<u16>) -> Self {
        let mt = unsafe {
            *(cpu
                .vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_String)
                .unwrap_class()
                .as_ref()
                .method_table())
        };

        const NUL: u16 = 0x0000;

        #[cfg(debug_assertions)]
        match bytes.iter().position(|&val| val == NUL) {
            None => (),
            Some(pos) if pos == bytes.len() - 1 => (),
            Some(pos) => {
                panic!("bytes contain NUL at {pos}, which is invalid in a C-like string");
            }
        }

        match bytes.last() {
            None => bytes.push(NUL),
            Some(&c) if c != NUL => bytes.push(NUL),
            Some(_) => (),
        }

        let this = {
            let layout = Layout::array::<u16>(bytes.len()).unwrap();
            let full_layout = Self::calc_full_layout(layout).unwrap();
            let ptr =
                std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, full_layout).unwrap();
            unsafe {
                ptr.cast::<ObjectHeader>().write(ObjectHeader::new(false));
                ptr.byte_add(offset_of!(ManagedReferenceInner<Class>, mt))
                    .cast::<NonNull<MethodTable<Class>>>()
                    .write(mt);
            }

            let ptr = ptr.cast();
            let this = Self { data: Some(ptr) };
            cpu.gen_mem_recorder(NonGenericTypeHandleKind::Class)(this);

            this
        };
        unsafe {
            this.data().unwrap().cast::<u16>().copy_from(
                NonNull::new_unchecked(bytes.as_ptr().cast_mut()),
                bytes.len(),
            );
        }

        this
    }
}

#[repr(transparent)]
pub struct StringAccessor(ManagedReference<Class>);

impl<T> IAccessor<T> for StringAccessor {
    #[inline(always)]
    default fn is_valid(_: &ManagedReference<T>) -> bool {
        false
    }
}

impl IAccessor<Class> for StringAccessor {
    fn is_valid(r: &ManagedReference<Class>) -> bool {
        r.method_table_ref()
            .and_then(|x| x.get_core_type_id())
            .is_some_and(|x| x == CoreTypeId::System_String)
    }
}

impl StringAccessor {
    pub fn to_string(&self) -> Result<Option<String>, widestring::error::Utf16Error> {
        self.get_str().map(U16CStr::to_string).transpose()
    }
    pub fn to_string_lossy(&self) -> Option<String> {
        self.get_str().map(U16CStr::to_string_lossy)
    }
    pub fn get_str(&self) -> Option<&U16CStr> {
        self.0
            .data()
            .map(|p| unsafe { U16CStr::from_ptr_str(p.cast::<u16>().as_ptr().cast_const()) })
    }

    /// Convert to MultiByte by [WideCharToMultiByte]
    ///
    /// Return value
    /// * Some((..., None)) if WideCharToMultiByte fails see [Return value]
    /// * [`None`] if self is null(could be checked by [`ManagedReference::is_null`]).
    ///
    /// [WideCharToMultiByte]: https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-widechartomultibyte
    /// [Return value]: https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-widechartomultibyte#return-value
    #[doc(cfg(windows))]
    #[cfg(windows)]
    pub fn to_multi_byte(&self) -> Option<(Vec<u8>, Option<std::num::NonZero<i32>>)> {
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

        let name_wide = self.get_str()?;

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

        let used_len_non_zero = std::num::NonZero::new(used_len);

        Some((name_out, used_len_non_zero))
    }
    /// With '\0' terminator, len in element
    pub fn raw_len(&self) -> Option<usize> {
        self.get_str().map(|x| x.len() + 1)
    }
}
