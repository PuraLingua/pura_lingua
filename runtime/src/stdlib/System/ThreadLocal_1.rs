use std::{cell::RefCell, collections::HashMap, ffi::c_void, ptr::NonNull};

use global::{ThreadSafe, attrs::CallConvention};

use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{
        class::Class,
        method::{ExceptionTable, Method},
        type_handle::{NonGenericTypeHandle, TypeHandle},
    },
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

#[cfg(test)]
mod tests;

#[derive(ThreadSafe)]
struct TlsEntry {
    ptr: NonNull<c_void>,
    ty: NonGenericTypeHandle,
}

impl Drop for TlsEntry {
    fn drop(&mut self) {
        unsafe {
            std::alloc::Allocator::deallocate(
                &std::alloc::System,
                self.ptr.cast(),
                self.ty.val_layout(),
            );
        }
    }
}

thread_local! {
    static HANDLES: RefCell<HashMap<ManagedReference<Class>, TlsEntry>> = RefCell::new(HashMap::new());
}

pub extern "system" fn Destructor(_: &mut CPU, _: &Method<Class>, this: &ManagedReference<Class>) {
    HANDLES.with_borrow_mut(|handles| {
        handles.remove(this);
    })
}

pub extern "system" fn Constructor(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) {
    let mt: &crate::type_system::method_table::MethodTable<Class> =
        unsafe { this.method_table_ref_unchecked() };

    let type_var_0 = mt.ty_ref().type_vars().as_ref().unwrap().get(0).unwrap();
    let ptr = match std::alloc::Allocator::allocate_zeroed(
        &std::alloc::System,
        type_var_0.val_layout(),
    ) {
        Ok(ptr) => ptr.as_non_null_ptr().cast::<c_void>(),
        Err(_) => {
            assert!(cpu.throw_helper_mut().alloc());
            return;
        }
    };
    let entry = TlsEntry {
        ptr,
        ty: *type_var_0,
    };
    HANDLES.with_borrow_mut(|handles| {
        handles.insert(*this, entry);
    })
}

pub extern "system" fn GetPointer(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
) -> NonNull<c_void> {
    HANDLES.with_borrow_mut(|handles| match handles.entry(*this) {
        std::collections::hash_map::Entry::Occupied(occupied_entry) => occupied_entry.get().ptr,
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            let mt: &crate::type_system::method_table::MethodTable<Class> =
                unsafe { this.method_table_ref_unchecked() };

            let type_var_0 = mt.ty_ref().type_vars().as_ref().unwrap().get(0).unwrap();

            let ptr = match std::alloc::Allocator::allocate_zeroed(
                &std::alloc::System,
                type_var_0.val_layout(),
            ) {
                Ok(ptr) => ptr.as_non_null_ptr().cast::<c_void>(),
                Err(_) => {
                    assert!(cpu.throw_helper_mut().alloc());
                    return NonNull::dangling();
                }
            };

            vacant_entry
                .insert(TlsEntry {
                    ptr,
                    ty: *type_var_0,
                })
                .ptr
        }
    })
}

pub extern "system" fn Get(
    _: &mut CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
    return_buffer: NonNull<c_void>,
) {
    let (ptr, ty) = HANDLES.with_borrow(|handles| {
        let entry = handles.get(this).unwrap();
        (entry.ptr, entry.ty)
    });

    unsafe {
        return_buffer
            .cast::<u8>()
            .copy_from(ptr.cast::<u8>(), ty.val_layout().size());
    }
}

macro define_registers($($name:ident)*) {$(
    const $name: global::instruction::ShortRegisterAddr = <global::instruction::ShortRegisterAddr as global::instruction::IRegisterAddr>::new(${index()});
)*}

_define_class!(
    fn load(assembly, mt, method_info)
    ThreadLocal_1
#methods(TMethodId):
    Destructor => common_new_method!(mt TMethodId Destructor Destructor);

    Constructor => common_new_method!(mt TMethodId Constructor Constructor);
    GetPointer => common_new_method!(mt TMethodId GetPointer GetPointer);
    Get => common_new_method!(mt TMethodId Get Get);
    Set => Method::new(
        mt,
        widestring::Utf16String::from_str(TMethodId::Set.get_name()),
        super::map_method_attr(TMethodId::Set.get_attr()),
        TMethodId::Set
            .get_generic_count()
            .map(From::from)
            .unwrap_or_default(),
        TMethodId::Set
            .get_parameters()
            .into_iter()
            .map(super::map_parameter)
            .collect(),
        TMethodId::Set.get_return_type().into(),
        CallConvention::PlatformDefault,
        None,
        {
            use global::instruction::{Instruction, Instruction_Load, Instruction_Call, CommonWritePointer, LoadContent};

            define_registers!(
                value
                size_of_value
                this
                pointer_to_storage
            );
            vec![
                Instruction::SLoad(Instruction_Load { addr: value, content: LoadContent::ArgValue(0) }),
                Instruction::SLoad(Instruction_Load {
                    addr: size_of_value,
                    content: LoadContent::TypeValueSize(TypeHandle::TypeGeneric(0).into()),
                }),
                Instruction::SLoad(Instruction_Load { addr: this, content: LoadContent::This }),

                Instruction::SCall(Instruction_Call::InstanceCall {
                    val: this,
                    method: TMethodId::GetPointer.into(),
                    args: vec![],
                    ret_at: pointer_to_storage,
                }),

                Instruction::SWritePointer(CommonWritePointer {
                    source: value,
                    size: size_of_value,
                    ptr: pointer_to_storage,
                }),
            ]
        },
        ExceptionTable::gen_new(),
    );
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
