use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Constructor(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
) {
    let message = ManagedReference::new_string(cpu, "Alloc failed");
    super::Exception::Constructor_String(cpu, method, this, message);
}

_define_class!(
    fn load(assembly, mt, method_info)
    AllocException
#methods(TMethodId):
    Constructor => common_new_method!(mt TMethodId Constructor Constructor);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
