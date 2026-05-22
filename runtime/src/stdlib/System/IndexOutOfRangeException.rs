use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub fn Constructor_USize_USize(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
    index: usize,
    length: usize,
) {
    let message = ManagedReference::new_string(
        cpu,
        &format!("Index out of range, the length is {length} while the index is {index}"),
    );
    super::Exception::Constructor_String(cpu, method, this, message);
}

super::_define_class!(
    fn load(assembly, mt, method_info)
    IndexOutOfRangeException
#methods(TMethodId):
    Constructor => super::common_new_method!(mt TMethodId Constructor Constructor_USize_USize);
#static_methods(TStaticMethodId):
    StaticConstructor => super::default_sctor!(mt TStaticMethodId);
);
