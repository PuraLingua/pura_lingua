use std::alloc::Layout;

use line_ending::LineEnding;

use crate::{
    stdlib::System_Environment_FieldId,
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn StaticConstructor(cpu: &CPU, method: &Method<Class>) {
    let (new_line_out, new_line_layout) = cpu
        .vm_ref()
        .get_static_field(
            method.require_method_table_ref().ty.into(),
            System_Environment_FieldId::NewLine as _,
        )
        .unwrap();

    debug_assert_eq!(new_line_layout, Layout::new::<ManagedReference<Class>>());
    unsafe {
        new_line_out
            .cast::<ManagedReference<Class>>()
            .write(ManagedReference::new_string(
                cpu,
                LineEnding::from_current_platform().as_str(),
            ))
    }
}
