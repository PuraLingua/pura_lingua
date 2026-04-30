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

    pub const fn empty() -> Self {
        Self {
            len: 0,
            ptr: NonNull::<T>::dangling().as_ptr(),
        }
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

#[repr(C)]
pub struct OwnedSlicePtr<T> {
    pub len: usize,
    pub ptr: *mut T,
}

impl<T> OwnedSlicePtr<T> {
    pub fn from_box(s: Box<[T]>) -> Self {
        Self {
            len: s.len(),
            ptr: Box::into_raw(s).cast(),
        }
    }
    pub fn from_vec(s: Vec<T>) -> Self {
        Self::from_box(s.into_boxed_slice())
    }

    pub const fn empty() -> Self {
        Self {
            len: 0,
            ptr: NonNull::<T>::dangling().as_ptr(),
        }
    }
    pub const fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }

    pub const fn into_option(self) -> Option<Self> {
        if self.len == 0 && self.ptr.is_null() {
            None
        } else {
            Some(Self {
                len: self.len,
                ptr: self.ptr,
            })
        }
    }

    /// # Safety
    ///
    /// the ptr must be valid
    pub unsafe fn into_vec(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.ptr, self.len, self.len) }
    }

    /// # Safety
    ///
    /// the ptr must be valid
    pub unsafe fn into_boxed_slice(self) -> Box<[T]> {
        unsafe { Box::from_raw(std::ptr::from_raw_parts_mut(self.ptr, self.len)) }
    }
}

impl<T> Deref for OwnedSlicePtr<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
    }
}

impl<T> DerefMut for OwnedSlicePtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
