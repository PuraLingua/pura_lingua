use std::ptr::NonNull;

use crate::{
    type_system::{
        method_table::MethodTable, r#struct::Struct, type_handle::NonGenericTypeHandleKind,
    },
    virtual_machine::cpu::{CPU, MemoryRecord},
};

use super::ManagedReference;

impl ManagedReference<Struct> {
    #[inline]
    pub fn common_alloc(cpu: &CPU, mt: NonNull<MethodTable<Struct>>, is_static: bool) -> Self {
        Self::base_common_alloc(mt, is_static, |ptr| {
            let mut records = cpu.write_mem_records().unwrap();
            records.push(MemoryRecord::new(
                NonGenericTypeHandleKind::Struct,
                ptr.cast(),
            ));
        })
    }
}
