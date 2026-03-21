use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn ToString(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) -> ManagedReference<Class> {
    ManagedReference::new_string(
        cpu,
        &this
            .access::<StringAccessor>()
            .unwrap()
            .to_string_lossy()
            .unwrap(),
    )
}

/// Returns the length of the string as number of elements (**not** number of bytes)
/// **not** including nul terminator.
pub extern "system" fn get_Length(
    _: &mut CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) -> usize {
    this.access::<StringAccessor>()
        .unwrap()
        .get_str()
        .unwrap()
        .len()
}

/// see [`self::get_Length`]
pub extern "system" fn get_U32Length(
    _: &mut CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) -> u32 {
    this.access::<StringAccessor>()
        .unwrap()
        .get_str()
        .unwrap()
        .len() as u32
}

_define_class!(
    fn load(assembly, mt, method_info)
    System_String
#methods(TMethodId):
    ToString => common_new_method!(mt TMethodId ToString ToString);
    get_Length => common_new_method!(mt TMethodId get_Length get_Length);
    get_U32Length => common_new_method!(mt TMethodId get_U32Length get_U32Length);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
