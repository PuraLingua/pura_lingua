use global::derive_ctor::ctor;

use crate::{
    type_system::type_handle::NonGenericTypeHandleKind, value::managed_reference::ManagedReference,
};

#[derive(Clone, Copy, ctor)]
#[ctor(pub new)]
pub struct MemoryRecord {
    kind: NonGenericTypeHandleKind,
    ptr: ManagedReference<()>,
}
