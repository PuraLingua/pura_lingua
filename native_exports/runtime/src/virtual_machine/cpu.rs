use std::{
    ffi::{CStr, c_char},
    ptr::NonNull,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use c_definitions::SlicePtr;
use pura_lingua::runtime::{
    type_system::{class::Class, method::Method},
    virtual_machine::cpu::{CPU, MainResult},
};

#[unsafe(no_mangle)]
pub extern "C" fn CPULock_Read<'a, 'lock: 'a>(
    lock: &'lock RwLock<CPU>,
) -> NonNull<RwLockReadGuard<'a, CPU>> {
    Box::into_non_null(Box::new(lock.read().unwrap()))
}
#[unsafe(no_mangle)]
pub extern "C" fn CPULock_Write<'a, 'lock: 'a>(
    lock: &'lock RwLock<CPU>,
) -> NonNull<RwLockWriteGuard<'a, CPU>> {
    Box::into_non_null(Box::new(lock.write().unwrap()))
}

#[unsafe(no_mangle)]
pub extern "C" fn CPULock_DropRead<'a>(ptr: NonNull<RwLockReadGuard<'a, CPU>>) {
    unsafe {
        drop(Box::from_non_null(ptr));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn CPULock_DropWrite<'a>(ptr: NonNull<RwLockWriteGuard<'a, CPU>>) {
    unsafe {
        drop(Box::from_non_null(ptr));
    }
}

/// Return true(1) if succeed, return false(0) and set last error otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn CPU_InvokeMain4Class(
    cpu: &mut CPU,
    method: &Method<Class>,
    args: SlicePtr<*const c_char>,
    result_out: NonNull<MainResult>,
) -> bool {
    let result = cpu.invoke_main(
        method,
        match args
            .iter()
            .map(|x| unsafe { CStr::from_ptr(*x).to_str().map(ToOwned::to_owned) })
            .try_collect()
        {
            Ok(args) => args,
            Err(e) => {
                crate::PURALINGUA_RUNTIME_ERROR.set(Some(e.into())).unwrap();
                return false;
            }
        },
    );
    unsafe {
        result_out.write(result);
    }
    true
}
