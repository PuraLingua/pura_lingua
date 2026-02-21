use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[repr(C)]
pub struct SlicePtr<T> {
    pub len: usize,
    pub ptr: *mut T,
}

impl<T> SlicePtr<T> {
    pub const fn from_slice(s: &[T]) -> Self {
        Self {
            len: s.len(),
            ptr: s.as_ptr().cast_mut(),
        }
    }

    pub fn from_box(s: Box<[T]>) -> Self {
        Self {
            len: s.len(),
            ptr: Box::into_raw(s).cast(),
        }
    }

    pub const fn empty() -> Self {
        Self {
            len: 0,
            ptr: NonNull::<T>::dangling().as_ptr(),
        }
    }

    /// # Safety
    ///
    /// the ptr refers to a full block(like the pointer returned by malloc)
    pub unsafe fn into_vec(self) -> Vec<T> {
        unsafe { self.into_boxed_slice().into_vec() }
    }

    /// # Safety
    ///
    /// the ptr refers to a full block(like the pointer returned by malloc)
    pub unsafe fn into_boxed_slice(self) -> Box<[T]> {
        unsafe { Box::from_raw(std::ptr::from_raw_parts_mut(self.ptr, self.len)) }
    }
}

impl<T> Deref for SlicePtr<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
    }
}

impl<T> DerefMut for SlicePtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
