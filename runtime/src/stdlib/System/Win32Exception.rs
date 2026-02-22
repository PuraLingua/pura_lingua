#[allow(unused)]
use crate::{
    stdlib::System_Win32Exception_FieldId,
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

#[cfg(windows)]
pub fn Constructor_I32(
    cpu: &CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
    code: i32,
) {
    assert!(
        this.const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_Win32Exception_FieldId::Code as _,
                Default::default(),
                code,
            )
    );

    super::Exception::Construct_String(
        cpu,
        method,
        this,
        ManagedReference::new_string(cpu, &windows::core::HRESULT(code).message()),
    );
}

#[cfg(not(windows))]
pub fn Constructor_I32(_: &CPU, _: &Method<Class>, _: &mut ManagedReference<Class>, _: i32) {
    panic!("Unsupported");
}
