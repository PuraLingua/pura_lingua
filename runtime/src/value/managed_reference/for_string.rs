use std::{mem::offset_of, ptr::NonNull, string::FromUtf16Error};

use crate::{
    stdlib::CoreTypeId,
    type_system::{class::Class, method_table::MethodTable},
    virtual_machine::cpu::CPU,
};

use super::{ArrayAccessor, IAccessor, ManagedReference, ManagedReferenceInner};

impl ManagedReference<Class> {
    #[track_caller]
    pub fn new_string(cpu: &CPU, mut s: String) -> Self {
        s.push('\0');
        let utf16 = s.encode_utf16().collect::<Vec<u16>>();
        let mut this = ManagedReference::alloc_array(
            cpu,
            *unsafe {
                cpu.vm_ref()
                    .assembly_manager()
                    .get_core_type(CoreTypeId::System_Char)
                    .unwrap_struct()
                    .as_ref()
                    .method_table()
            },
            utf16.len(),
        );

        unsafe {
            this.data
                .unwrap()
                .byte_add(offset_of!(ManagedReferenceInner<Class>, mt))
                .cast::<NonNull<MethodTable<Class>>>()
                .write(
                    *(cpu
                        .vm_ref()
                        .assembly_manager()
                        .get_core_type(CoreTypeId::System_String)
                        .unwrap_class()
                        .as_ref()
                        .method_table()),
                );
        }

        let dest = unsafe {
            this.access_unchecked_mut::<ArrayAccessor>()
                .as_slice_mut::<u16>()
                .unwrap()
        };
        dest.copy_from_slice(&utf16);

        this
    }
}

#[repr(transparent)]
pub struct StringAccessor(ManagedReference<Class>);

impl IAccessor<Class> for StringAccessor {
    fn is_valid(r: &ManagedReference<Class>) -> bool {
        r.method_table_ref()
            .and_then(|x| x.get_core_type_id())
            .is_some_and(|x| x == CoreTypeId::System_String)
    }
}

impl StringAccessor {
    pub fn to_string(&self) -> Result<Option<String>, FromUtf16Error> {
        unsafe { self.0.access_unchecked::<ArrayAccessor>().as_slice::<u16>() }
            .map(|x| &x[..(x.len() - 1)]) // Remove '\0'
            .map(String::from_utf16)
            .transpose()
    }
    pub fn to_string_lossy(&self) -> Option<String> {
        unsafe { self.0.access_unchecked::<ArrayAccessor>().as_slice::<u16>() }
            .map(|x| &x[..(x.len() - 1)]) // Remove '\0'
            .map(String::from_utf16_lossy)
    }
    /// With '\0' terminator, len in u16
    pub fn raw_len(&self) -> Option<usize> {
        // Safety: String and Array share the same layout
        unsafe { self.0.access_unchecked::<ArrayAccessor>().len() }
    }
}
