use std::ptr::NonNull;

use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Initialize(
    _: &CPU,
    _: &Method<Class>,
    this: NonNull<u8>,
    val: ManagedReference<Class>,
) {
    assert!(this.is_aligned_to(align_of::<ManagedReference<Class>>()));
    unsafe {
        this.cast::<ManagedReference<Class>>().write(val);
    }
}
