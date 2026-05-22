#[cfg(windows)]
use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

#[cfg(windows)]
pub fn Constructor(cpu: &mut CPU, method: &Method<Class>, this: &mut ManagedReference<Class>) {
    let name = ManagedReference::new_string(cpu, "<unnamed>");
    Constructor_String(cpu, method, this, name);
}

#[cfg(windows)]
pub fn Constructor_String(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
    name: ManagedReference<Class>,
) {
    let message = ManagedReference::new_string(
        cpu,
        &format!(
            "Try to dereference NULL ptr {}",
            name.access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap()
                .display()
        ),
    );
    super::Exception::Constructor_String(cpu, method, this, message);
}

super::_define_class!(
    fn load(assembly, mt, method_info)
    NullReferenceException
#methods(TMethodId):
    Constructor => super::common_new_method!(mt TMethodId Constructor Constructor);
    Constructor_String => super::common_new_method!(mt TMethodId Constructor_String Constructor_String);
#static_methods(TStaticMethodId):
    StaticConstructor => super::default_sctor!(mt TStaticMethodId);
);
