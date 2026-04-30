use std::{
    alloc::{Layout, LayoutError},
    ptr::NonNull,
};

#[repr(C)]
#[derive(Copy)]
#[derive_const(Clone)]
pub struct BasicVTable {
    pub size: usize,
    pub align: usize,
    pub drop: extern "system" fn(NonNull<u8>),
}

impl BasicVTable {
    pub const fn new<T>() -> Self {
        extern "system" fn _drop<T>(this: NonNull<u8>) {
            unsafe {
                this.cast::<T>().drop_in_place();
            }
        }

        Self {
            size: size_of::<T>(),
            align: align_of::<T>(),
            drop: _drop::<T>,
        }
    }

    pub const fn layout(&self) -> Result<Layout, LayoutError> {
        Layout::from_size_align(self.size, self.align)
    }
}
