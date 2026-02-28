use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

pub fn Constructor_String_String(
    cpu: &CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
    enum_name: ManagedReference<Class>,
    message: ManagedReference<Class>,
) {
    super::Exception::Constructor_String(
        cpu,
        method,
        this,
        ManagedReference::new_string(
            cpu,
            &format!(
                "Enum {} of certain value is invalid: {}",
                enum_name
                    .access::<StringAccessor>()
                    .unwrap()
                    .get_str()
                    .unwrap()
                    .display(),
                message
                    .access::<StringAccessor>()
                    .unwrap()
                    .get_str()
                    .unwrap()
                    .display(),
            ),
        ),
    );
}
