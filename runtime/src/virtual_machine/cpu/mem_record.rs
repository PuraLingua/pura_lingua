use derive_more::Display;
use global::derive_ctor::ctor;

use crate::{
    type_system::type_handle::NonGenericTypeHandleKind, value::managed_reference::ManagedReference,
};

use super::CPU;

#[derive(Clone, Copy, Debug, Display, ctor)]
#[ctor(pub new)]
#[display(
    "MemoryRecord {{ptr: {} of {}, to_be_dropped: {}}}",
    ptr,
    kind,
    to_be_dropped
)]
pub struct MemoryRecord {
    pub(crate) kind: NonGenericTypeHandleKind,
    pub(crate) ptr: ManagedReference<()>,
    #[ctor(expr(false))]
    pub(crate) to_be_dropped: bool,
}

impl CPU {
    /// Used for [`crate::value::managed_reference::ManagedReference::base_common_alloc`]
    pub fn gen_mem_recorder<T>(
        &self,
        kind: NonGenericTypeHandleKind,
    ) -> impl FnOnce(ManagedReference<T>) {
        move |ptr| {
            let mut records = self.write_mem_records().unwrap();
            records.push(MemoryRecord::new(kind, ptr.cast()));
        }
    }
}
