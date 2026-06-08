use std::ptr::NonNull;

use global::ThreadSafe;

#[derive(ThreadSafe)]
pub struct ThreadSafePtr<T: ?Sized>(pub *const T);

#[derive(ThreadSafe)]
pub struct ThreadSafeMutPtr<T: ?Sized>(pub *mut T);

#[derive(ThreadSafe)]
pub struct ThreadSafeNonNull<T: ?Sized>(pub NonNull<T>);

macro impl_ptr($TGeneric:ident $Ty:ident: $getter:expr; $($items:item)*) {
    impl<$TGeneric: ?Sized> $Ty<$TGeneric> {
		$($items)*
	}

    const impl<$TGeneric: ?Sized> Clone for $Ty<$TGeneric> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<$TGeneric: ?Sized> Copy for $Ty<$TGeneric> {}

    impl<$TGeneric: ?Sized> std::hash::Hash for $Ty<$TGeneric> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            let getter: impl FnOnce(&Self) -> *const $TGeneric = $getter;
            std::ptr::hash((getter)(self), state);
        }
    }

    impl<$TGeneric: ?Sized> PartialEq for $Ty<$TGeneric> {
        fn eq(&self, other: &Self) -> bool {
            let getter: impl Fn(&Self) -> *const $TGeneric = $getter;
            std::ptr::addr_eq((getter)(self), (getter)(other))
        }
    }

    impl<T: ?Sized> Eq for $Ty<T> {}
}

impl_ptr!(
    T
    ThreadSafePtr: |this| this.0;
    pub const fn new(ptr: *const T) -> Self {
        Self(ptr)
    }
    pub const unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        unsafe { self.0.as_ref() }
    }
    pub const unsafe fn as_ref_unchecked<'a>(self) -> &'a T {
        unsafe { self.0.as_ref_unchecked() }
    }
);
impl_ptr!(
    T
    ThreadSafeMutPtr: |this| this.0;
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }
    pub const unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        unsafe { self.0.as_ref() }
    }
    pub const unsafe fn as_ref_unchecked<'a>(self) -> &'a T {
        unsafe { self.0.as_ref_unchecked() }
    }
    pub const unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
        unsafe { self.0.as_mut() }
    }
    pub const unsafe fn as_mut_unchecked<'a>(self) -> &'a mut T {
        unsafe { self.0.as_mut_unchecked() }
    }
);
impl_ptr!(
    T
    ThreadSafeNonNull: |this| this.0.as_ptr();
    pub const fn new(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }
    pub const unsafe fn as_ref<'a>(&self) -> &'a T {
        unsafe { self.0.as_ref() }
    }
    pub const unsafe fn as_mut<'a>(&mut self) -> &'a mut T {
        unsafe { self.0.as_mut() }
    }
);
