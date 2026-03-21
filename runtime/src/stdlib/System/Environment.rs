use std::alloc::Layout;

use line_ending::LineEnding;
use stdlib_header::definitions::System_Environment_FieldId;

use crate::{
    stdlib::System::_define_class,
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn StaticConstructor(cpu: &mut CPU, method: &Method<Class>) {
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

_define_class!(
    fn load(assembly, mt, method_info)
    System_Environment
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => Box::new(Method::create_sctor(
        Some(mt),
        super::map_method_attr(TStaticMethodId::StaticConstructor.get_attr()),
        StaticConstructor,
    ));
);
