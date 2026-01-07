use std::{alloc::Layout, ptr::NonNull};

use crate::{
    memory::GetFieldOffsetOptions,
    type_system::{class::Class, method_table::MethodTable, type_handle::NonGenericTypeHandleKind},
    virtual_machine::cpu::{CPU, MemoryRecord},
};

use super::ManagedReference;

impl ManagedReference<Class> {
    #[inline]
    pub fn common_alloc(cpu: &CPU, mt: NonNull<MethodTable<Class>>, is_static: bool) -> Self {
        Self::base_common_alloc(mt, is_static, |ptr| {
            let mut records = cpu.write_mem_records().unwrap();
            records.push(MemoryRecord::new(
                NonGenericTypeHandleKind::Class,
                ptr.cast(),
            ));
        })
    }
}

impl ManagedReference<Class> {
    pub fn field(
        &self,
        is_static: bool,
        i: u32,
        options: GetFieldOffsetOptions,
    ) -> Option<(NonNull<()>, Layout)> {
        let mt = self.method_table_ref()?;
        let offset = if is_static {
            mt.static_field_offset(i, options)?
        } else {
            mt.field_offset(i, options)?
        };

        let f_ptr = unsafe { self.data()?.byte_add(offset) };
        let f_layout =
            mt.ty_ref().fields()[i as usize].layout_with_type(mt.ty_ref(), Default::default());

        Some((f_ptr, f_layout))
    }

    pub fn typed_field<T>(
        &self,
        is_static: bool,
        i: u32,
        options: GetFieldOffsetOptions,
    ) -> Option<&T> {
        let (f_ptr, layout) = self.field(is_static, i, options)?;
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        Some(unsafe { f_ptr.cast::<T>().as_ref() })
    }

    pub fn typed_field_mut<T>(
        &mut self,
        is_static: bool,
        i: u32,
        options: GetFieldOffsetOptions,
    ) -> Option<&mut T> {
        let (f_ptr, layout) = self.field(is_static, i, options)?;
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        Some(unsafe { f_ptr.cast::<T>().as_mut() })
    }
}
