use std::alloc::Layout;

use crate::{
    stdlib::System::_define_struct,
    type_system::{method::Method, r#struct::Struct},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn StaticConstructor(cpu: &mut CPU, method: &Method<Struct>) {
    let (null_ptr, null_layout) = cpu
        .vm_ref()
        .get_static_field(
            method.require_method_table_ref().ty.into(),
            stdlib_header::System::Pointer::FieldId::Null as u32,
        )
        .unwrap();
    debug_assert_eq!(null_layout, Layout::new::<*const u8>());
    unsafe {
        null_ptr.cast::<*const u8>().write(std::ptr::null());
    }
}

_define_struct!(
    fn load(assembly, mt, method_info)
    Pointer
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => Method::create_sctor(
        Some(mt),
        super::map_method_attr(TStaticMethodId::StaticConstructor.get_attr()),
        StaticConstructor,
    );
);
