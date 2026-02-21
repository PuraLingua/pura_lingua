use std::alloc::Layout;

use crate::{
    stdlib::System_Pointer_FieldId,
    type_system::{method::Method, r#struct::Struct},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn StaticConstructor(cpu: &CPU, method: &Method<Struct>) {
    let (null_ptr, null_layout) = cpu
        .vm_ref()
        .get_static_field(
            method.require_method_table_ref().ty.into(),
            System_Pointer_FieldId::Null as u32,
        )
        .unwrap();
    debug_assert_eq!(null_layout, Layout::new::<*const u8>());
    unsafe {
        null_ptr.cast::<*const u8>().write(std::ptr::null());
    }
}
