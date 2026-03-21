use std::ptr::NonNull;

use pura_lingua::runtime::type_system::assembly::Assembly;

#[unsafe(no_mangle)]
pub extern "C" fn Assembly_Drop(this: NonNull<Assembly>) {
    unsafe {
        drop(Box::from_non_null(this));
    }
}
