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

#[doc(hidden)]
pub struct MemoryRecorder<'a>(&'a mut CPU, NonGenericTypeHandleKind);

impl<'a, T> FnOnce<(ManagedReference<T>,)> for MemoryRecorder<'a> {
    type Output = ();
    extern "rust-call" fn call_once(self, (ptr,): (ManagedReference<T>,)) -> Self::Output {
        self.0
            .mem_records
            .push(MemoryRecord::new(self.1, ptr.cast()));
    }
}

impl CPU {
    /// Used for [`crate::value::managed_reference::ManagedReference::base_common_alloc`]
    pub const fn gen_mem_recorder<'a>(
        &'a mut self,
        kind: NonGenericTypeHandleKind,
    ) -> MemoryRecorder<'a> {
        MemoryRecorder(self, kind)
    }
}
