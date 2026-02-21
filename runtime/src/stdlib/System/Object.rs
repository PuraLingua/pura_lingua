use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Destructor(_: &CPU, _: &Method<Class>, _: &ManagedReference<Class>) {
    println!("DEFAULT Destructor");
}

pub extern "system" fn ToString(
    cpu: &CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) -> ManagedReference<Class> {
    println!("DEFAULT ToString");
    match this.method_table_ref() {
        Some(mt) => {
            let name = mt.ty_ref().name();
            ManagedReference::new_string(cpu, name.as_str())
        }
        None => ManagedReference::new_string(cpu, "<UNKNOWN TYPE>"),
    }
}
