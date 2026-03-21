use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::{LargeStringAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn ToString(
    cpu: &mut CPU,
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

_define_class!(
    fn load(assembly, mt, method_info)
    System_LargeString
#methods(TMethodId):
    ToString => common_new_method!(mt TMethodId ToString ToString);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
