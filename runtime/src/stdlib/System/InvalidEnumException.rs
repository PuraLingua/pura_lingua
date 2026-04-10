use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

pub fn Constructor_String_String(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
    enum_name: ManagedReference<Class>,
    message: ManagedReference<Class>,
) {
    let message = ManagedReference::new_string(
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
    );
    super::Exception::Constructor_String(cpu, method, this, message);
}

_define_class!(
    fn load(assembly, mt, method_info)
    InvalidEnumException
#methods(TMethodId):
    Constructor_String_String => common_new_method!(mt TMethodId Constructor_String_String Constructor_String_String);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
