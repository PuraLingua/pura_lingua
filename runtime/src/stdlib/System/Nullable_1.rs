use std::ptr::NonNull;

use crate::{
    stdlib::System::common_new_method,
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Initialize(
    _: &CPU,
    _: &Method<Class>,
    this: NonNull<u8>,
    val: ManagedReference<Class>,
) {
    assert!(this.is_aligned_to(align_of::<ManagedReference<Class>>()));
    unsafe {
        this.cast::<ManagedReference<Class>>().write(val);
    }
}

use crate::stdlib::System::{_define_struct, default_sctor};

_define_struct!(
    fn load(assembly, mt, method_info)
    Nullable_1
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
    Initialize => common_new_method!(mt TStaticMethodId Initialize Initialize);
);
