use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::{LargeStringAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn ToString(
    cpu: &CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) -> ManagedReference<Class> {
    ManagedReference::new_string(
        cpu,
        this.access::<LargeStringAccessor>()
            .unwrap()
            .as_str()
            .unwrap(),
    )
}
