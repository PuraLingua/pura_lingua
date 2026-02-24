// #[allow(unused)]
use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

#[cfg(unix)]
pub fn Constructor(cpu: &CPU, method: &Method<Class>, this: &mut ManagedReference<Class>) {
    Constructor_I32(cpu, method, this, unsafe {
        *crate::virtual_machine::cpu::errno_location()
    });
}

#[cfg(unix)]
pub fn Constructor_I32(
    cpu: &CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
    code: libc::c_int,
) {
    use std::ffi::CString;

    use crate::{stdlib::System_ErrnoException_FieldId, value::managed_reference::FieldAccessor};

    assert!(
        this.const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_ErrnoException_FieldId::Code as _,
                Default::default(),
                code,
            )
    );

    super::Exception::Construct_String(
        cpu,
        method,
        this,
        ManagedReference::new_string(
            cpu,
            // cSpell:disable-next-line
            &unsafe { CString::from_raw(libc::strerror(code)) }
                .into_string()
                .unwrap(),
        ),
    );
}

#[cfg(not(unix))]
pub fn Constructor(_: &CPU, _: &Method<Class>, _: &mut ManagedReference<Class>) {
    panic!("Unsupported");
}

#[cfg(not(unix))]
pub fn Constructor_I32(
    _: &CPU,
    _: &Method<Class>,
    _: &mut ManagedReference<Class>,
    _: std::ffi::c_int,
) {
    panic!("Unsupported");
}
