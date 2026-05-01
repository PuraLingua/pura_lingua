use crate::{
    stdlib::System::{_define_interface, common_new_method},
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Dispose(_: &mut CPU, _: &Method<Class>, _: &mut ManagedReference<Class>) {
    unimplemented!()
}

_define_interface!(
    fn load(assembly, mt, method_info)
    IDispose
#methods(TMethodId):
    Dispose => common_new_method!(mt TMethodId Dispose Dispose);
);
