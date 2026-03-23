use std::ffi::c_void;
use std::ptr::NonNull;

use global::t_println;
use stdlib_header::definitions::System_DynamicLibrary_FieldId;

use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

#[cfg(test)]
mod tests;

pub extern "system" fn Constructor_String(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    file: ManagedReference<Class>,
) {
    t_println!(
        "Loading lib: {}",
        file.access::<StringAccessor>()
            .unwrap()
            .get_str()
            .unwrap()
            .display()
    );
    let handle_out = this
        .const_access_mut::<FieldAccessor<Class>>()
        .typed_field_mut::<LibraryPointer>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    let Some(handle) = LoadLibraryImpl(cpu, file) else {
        return;
    };
    *handle_out = handle;
    println!("finish loading successfully");
}

pub extern "system" fn GetSymbol(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    let handle = this
        .const_access_mut::<FieldAccessor<Class>>()
        .read_typed_field::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    GetSymbolImpl(cpu, handle, name)
}

pub extern "system" fn Destructor(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
) {
    let handle = this
        .const_access_mut::<FieldAccessor<Class>>()
        .read_typed_field::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    if !FreeLibraryImpl(cpu, handle) {
        return;
    }
    ClearHandlePtr(this);
}

fn ClearHandlePtr(this: &mut ManagedReference<Class>) {
    assert!(
        this.const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field::<*mut c_void>(
                System_DynamicLibrary_FieldId::Handle as _,
                Default::default(),
                std::ptr::null_mut()
            )
    );
}

mod r#impl;

use r#impl::*;

_define_class!(
    fn load(assembly, mt, method_info)
    System_DynamicLibrary
#methods(TMethodId):
    Destructor => common_new_method!(mt TMethodId Destructor Destructor);
    Constructor_String => common_new_method!(mt TMethodId Constructor_String Constructor_String);
    GetSymbol => common_new_method!(mt TMethodId GetSymbol GetSymbol);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
