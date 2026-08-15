use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn FreeObject(
    cpu: &mut CPU,
    _method: &Method<Class>,
    mut obj: ManagedReference<Class>,
) {
    #[cfg(feature = "print_invoke_and_call")]
    eprintln!(
        "Freeing {:p}",
        obj.data
            .map(std::ptr::NonNull::as_ptr)
            .unwrap_or(std::ptr::null_mut())
    );
    obj.destroy(cpu);
}

_define_class!(
    fn load(assembly, mt, method_info)
    MemoryManagement
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
    FreeObject => common_new_method!(mt TStaticMethodId FreeObject FreeObject);
);
