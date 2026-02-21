use std::{alloc::Layout, mem::offset_of, ptr::NonNull};

use stdlib_header::CoreTypeId;

use crate::{
    type_system::{class::Class, method_table::MethodTable, type_handle::NonGenericTypeHandleKind},
    value::{managed_reference::ManagedReferenceInner, object_header::ObjectHeader},
    virtual_machine::cpu::CPU,
};

use super::{IAccessor, ManagedReference};

#[repr(transparent)]
pub struct LargeStringAccessor(ManagedReference<Class>);

impl<T> IAccessor<T> for LargeStringAccessor {
    #[inline(always)]
    default fn is_valid(_: &ManagedReference<T>) -> bool {
        false
    }
}

impl IAccessor<Class> for LargeStringAccessor {
    fn is_valid(r: &ManagedReference<Class>) -> bool {
        r.method_table_ref()
            .and_then(|x| x.get_core_type_id())
            .is_some_and(|x| x == CoreTypeId::System_LargeString)
    }
}

impl ManagedReference<Class> {
    pub fn new_large_string(cpu: &CPU, s: &str) -> Self {
        let mt = unsafe {
            *(cpu
                .vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_LargeString)
                .unwrap_class()
                .as_ref()
                .method_table())
        };

        let mut layout = Layout::new::<usize>();
        let data_offset;
        (layout, data_offset) = layout
            .extend(Layout::array::<u8>(s.len()).unwrap())
            .unwrap();

        let this = {
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
            this.data_ptr().cast::<usize>().write(s.len());
            this.data_ptr()
                .byte_add(data_offset)
                .cast::<u8>()
                .copy_from(s.as_ptr(), s.len());
        }

        this
    }
}

impl LargeStringAccessor {
    pub fn as_str(&self) -> Option<&str> {
        self.0.data().map(|p| unsafe {
            let len = p.cast::<usize>().read();
            const OFFSET: usize = Layout::new::<usize>()
                .extend(Layout::new::<u8>())
                .ok()
                .unwrap()
                .1;
            &*std::ptr::from_raw_parts(p.cast::<u8>().byte_add(OFFSET).as_ptr().cast_const(), len)
        })
    }
}
