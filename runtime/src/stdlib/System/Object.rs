use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Destructor(_: &mut CPU, _: &Method<Class>, _: &ManagedReference<Class>) {
    println!("DEFAULT Destructor");
}

pub extern "system" fn ToString(
    cpu: &mut CPU,
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

_define_class!(
    fn load(assembly, mt, method_info)
    Object
#methods(TMethodId):
    Destructor => common_new_method!(
        mt TMethodId Destructor Destructor
    );
    ToString => common_new_method!(
        mt TMethodId ToString ToString
    );
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
