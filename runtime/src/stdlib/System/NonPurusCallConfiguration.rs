use stdlib_header::System::NonPurusCallConfiguration::FieldId;

use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Constructor(
    _: &CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    call_convention: u8,
    return_type: ManagedReference<Class>,
    encoding: u8,
    object_strategy: u8,
    by_ref_arguments: ManagedReference<Class>,
    arguments: ManagedReference<Class>,
) {
    let accessor = this.const_access_mut::<FieldAccessor<Class>>();
    *accessor
        .typed_field_mut(FieldId::CallConvention as _, Default::default())
        .unwrap() = call_convention;

    *accessor
        .typed_field_mut(FieldId::ReturnType as _, Default::default())
        .unwrap() = return_type;

    *accessor
        .typed_field_mut(FieldId::Encoding as _, Default::default())
        .unwrap() = encoding;

    *accessor
        .typed_field_mut(FieldId::ObjectStrategy as _, Default::default())
        .unwrap() = object_strategy;

    *accessor
        .typed_field_mut(FieldId::ByRefArguments as _, Default::default())
        .unwrap() = by_ref_arguments;

    *accessor
        .typed_field_mut(FieldId::Arguments as _, Default::default())
        .unwrap() = arguments;
}

_define_class!(
    fn load(assembly, mt, method_info)
    NonPurusCallConfiguration
#methods(TMethodId):
    Constructor => common_new_method!(mt TMethodId Constructor Constructor);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
