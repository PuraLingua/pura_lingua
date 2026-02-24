use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn ToString(
    cpu: &CPU,
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
    _: &CPU,
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
    _: &CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) -> u32 {
    this.access::<StringAccessor>()
        .unwrap()
        .get_str()
        .unwrap()
        .len() as u32
}
