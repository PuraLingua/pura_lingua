use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{BasicVTable, new_ref_vtable};

#[repr(C)]
#[derive(Copy)]
#[derive_const(Clone)]
pub struct CAnyPtr {
    pub ptr: NonNull<u8>,
    pub vtable: &'static CAnyVTable,
}

#[repr(C)]
#[derive(Copy)]
#[derive_const(Clone)]
pub struct CAnyVTable {
    pub basic: BasicVTable,

    pub clone_into: Option<extern "C" fn(this: NonNull<u8>, out: NonNull<u8>)>,
}

impl CAnyVTable {
    pub const fn new<T>() -> Self {
        const trait MaybeClone {
            fn get_clone_into() -> Option<extern "C" fn(this: NonNull<u8>, out: NonNull<u8>)>;
        }

        const impl<T> MaybeClone for T {
            #[inline(always)]
            default fn get_clone_into() -> Option<extern "C" fn(this: NonNull<u8>, out: NonNull<u8>)>
            {
                None
            }
        }

        const impl<T: Clone> MaybeClone for T {
            #[inline(always)]
            fn get_clone_into() -> Option<extern "C" fn(this: NonNull<u8>, out: NonNull<u8>)> {
                extern "C" fn clone<T: Clone>(this: NonNull<u8>, out: NonNull<u8>) {
                    let this: &T = unsafe { this.cast::<T>().as_ref() };
                    unsafe {
                        out.cast::<T>().write(this.clone());
                    }
                }
                Some(clone::<T>)
            }
        }

        Self {
            basic: BasicVTable::new::<T>(),

            clone_into: T::get_clone_into(),
        }
    }

    #[inline(always)]
    pub const fn new_ref<T>() -> &'static Self {
        new_ref_vtable(Self::new::<T>)
    }
}

#[repr(transparent)]
pub struct BoxedCAny(CAnyPtr);

impl BoxedCAny {
    pub fn new<T>(val: T) -> Self {
        let vtable = const { CAnyVTable::new_ref::<T>() };
        // SAFETY: T's layout is always valid
        let ptr = match std::alloc::Allocator::allocate(&std::alloc::System, unsafe {
            vtable.basic.layout_unchecked()
        }) {
            Ok(x) => x.as_non_null_ptr(),
            Err(_) => {
                std::alloc::handle_alloc_error(unsafe { vtable.basic.layout_unchecked() });
            }
        };
        unsafe {
            ptr.cast::<T>().write(val);
        }
        Self(CAnyPtr { ptr, vtable })
    }

    /// # Safety
    /// ptr may be valid
    #[inline]
    pub const unsafe fn from_ptr(ptr: CAnyPtr) -> Self {
        Self(ptr)
    }

    #[inline]
    pub fn into_ptr(self) -> CAnyPtr {
        let man = ManuallyDrop::new(self);
        man.0
    }

    #[inline]
    pub fn as_ptr(&self) -> CAnyPtr {
        self.0
    }

    pub fn try_clone(&self) -> Result<Self, ()> {
        let Some(clone_into) = self.0.vtable.clone_into else {
            return Err(());
        };

        let layout = self.0.vtable.basic.layout().unwrap();
        let out = match std::alloc::Allocator::allocate(&std::alloc::System, layout) {
            Ok(x) => x.as_non_null_ptr(),
            Err(_) => {
                std::alloc::handle_alloc_error(layout);
            }
        };

        clone_into(self.0.ptr, out);

        Ok(Self(CAnyPtr {
            ptr: out,
            vtable: self.0.vtable,
        }))
    }
}

impl Drop for BoxedCAny {
    fn drop(&mut self) {
        (self.0.vtable.basic.drop)(self.0.ptr);
        unsafe {
            std::alloc::Allocator::deallocate(
                &std::alloc::System,
                self.0.ptr,
                self.0.vtable.basic.layout().unwrap(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::DropGuard;

    use super::*;

    #[test]
    fn test_c_any() {
        let b1 = BoxedCAny::new(DropGuard::new(0, |x| println!("{x}")));
        let b2 = BoxedCAny::new(DropGuard::new(1, |x| println!("{x}")));
        let b3 = BoxedCAny::new(DropGuard::new(2, |x| println!("{x}")));

        drop(b1);
        drop(b3);
        drop(b2);

        let clone1 = BoxedCAny::new(vec![10, 20, 30]);
        let clone2 = clone1.try_clone().unwrap();
        let clone3 = clone1.try_clone().unwrap();

        drop(clone1);
        drop(clone2);
        drop(clone3);
    }
}
