use std::{ffi::c_void, ptr::NonNull};

use global::instruction::{CommonReadPointerTo, CommonWritePointer, IRegisterAddr};

use crate::{
    type_system::{
        get_traits::{GetAssemblyRef, GetTypeVars},
        method::{
            Method,
            default_entry_point::{Termination, call_frame, load_register_failed},
        },
    },
    virtual_machine::cpu::CPU,
};

pub(super) fn read_pointer_to<
    T: Sized + GetAssemblyRef + GetTypeVars,
    TRegisterAddr: IRegisterAddr,
>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    ins: &CommonReadPointerTo<TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    let Some(&ptr_var) = call_frame(cpu).get_typed::<*const u8, _>(ins.ptr) else {
        load_register_failed!(ins.ptr);
    };
    if ptr_var.is_null() {
        return Some(Err(Termination::NullReference(
            core::panic::Location::caller(),
        )));
    }
    let Some(size) = call_frame(cpu).get_typed(ins.size) else {
        load_register_failed!(ins.size);
    };
    let Some(destination) = call_frame(cpu).get(ins.destination) else {
        load_register_failed!(ins.destination);
    };
    unsafe {
        ptr_var.copy_to(destination.ptr.as_ptr(), *size);
    }
    Some(Ok(()))
}

pub(super) fn write_pointer<
    T: Sized + GetAssemblyRef + GetTypeVars,
    TRegisterAddr: IRegisterAddr,
>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    ins: &CommonWritePointer<TRegisterAddr>,
) -> Option<Result<(), Termination>> {
    let Some(source) = call_frame(cpu).get(ins.source) else {
        load_register_failed!(ins.source);
    };
    let Some(size) = call_frame(cpu).get_typed::<usize, _>(ins.size) else {
        load_register_failed!(ins.size);
    };
    let Some(&ptr_var) = call_frame(cpu).get_typed::<*const u8, _>(ins.ptr) else {
        load_register_failed!(ins.ptr);
    };
    let Some(ptr_var) = NonNull::new(ptr_var.cast_mut()) else {
        return Some(Err(Termination::NullReference(
            core::panic::Location::caller(),
        )));
    };
    unsafe {
        source.copy_to(ptr_var, *size);
    }
    Some(Ok(()))
}
