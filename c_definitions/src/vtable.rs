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
    pub drop: extern "C" fn(NonNull<u8>),
}

impl BasicVTable {
    pub const fn new<T>() -> Self {
        extern "C" fn _drop<T>(this: NonNull<u8>) {
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
    /// Creates a layout, bypassing all checks.
    ///
    /// # Safety
    ///
    /// This function is unsafe as it does not verify the preconditions from
    /// [`Layout::from_size_align`].
    pub const unsafe fn layout_unchecked(&self) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.size, self.align) }
    }
}

#[inline(always)]
#[track_caller]
pub const fn new_ref_vtable<VTable, FCreate: const FnOnce() -> VTable>(
    create: FCreate,
) -> &'static VTable {
    unsafe {
        let this = std::intrinsics::const_allocate(size_of::<VTable>(), align_of::<VTable>());
        if this.is_null() {
            panic!("IT SHOULD BE USED IN CONST");
        }
        this.cast::<VTable>().write(create());
        &*std::intrinsics::const_make_global(this).cast::<VTable>()
    }
}
