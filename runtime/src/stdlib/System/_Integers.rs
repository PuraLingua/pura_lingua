use std::fmt::Display;

use crate::{
    type_system::{class::Class, method::Method, r#struct::Struct},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn ToString<T: Display>(
    cpu: &CPU,
    _: &Method<Struct>,
    this: &T,
) -> ManagedReference<Class> {
    ManagedReference::new_string(cpu, this.to_string())
}
