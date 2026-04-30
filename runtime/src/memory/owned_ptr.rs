//! From [`std::ptr::Unique`]

use std::{
    clone::TrivialClone,
    fmt,
    marker::{PhantomData, PointeeSized, Unsize},
    ops::{CoerceUnsized, DispatchFromDyn},
    ptr::NonNull,
};

/// A wrapper around a raw non-null `*mut T` that indicates that the possessor
/// of this wrapper owns the referent. Useful for building abstractions like
/// `Box<T>`, `Vec<T>`, `String`, and `HashMap<K, V>`.
///
/// Unlike `*mut T`, `OwnedPtr<T>` behaves "as if" it were an instance of `T`.
/// It implements `Send`/`Sync` if `T` is `Send`/`Sync`. It also implies
/// the kind of strong aliasing guarantees an instance of `T` can expect:
/// the referent of the pointer should not be modified without a unique path to
/// its owning OwnedPtr.
///
/// If you're uncertain of whether it's correct to use `OwnedPtr` for your purposes,
/// consider using `NonNull`, which has weaker semantics.
///
/// Unlike `*mut T`, the pointer must always be non-null, even if the pointer
/// is never dereferenced. This is so that enums may use this forbidden value
/// as a discriminant -- `Option<OwnedPtr<T>>` has the same size as `OwnedPtr<T>`.
/// However the pointer may still dangle if it isn't dereferenced.
///
/// Unlike `*mut T`, `OwnedPtr<T>` is covariant over `T`. This should always be correct
/// for any type which upholds OwnedPtr's aliasing requirements.
#[repr(transparent)]
pub struct OwnedPtr<T: PointeeSized> {
    pointer: NonNull<T>,
    // NOTE: this marker has no consequences for variance, but is necessary
    // for drop checker to understand that we logically own a `T`.
    //
    // For details, see:
    // https://github.com/rust-lang/rfcs/blob/master/text/0769-sound-generic-drop.md#phantom-data
    _marker: PhantomData<T>,
}

/// Layout assertions
const _: () = {
    assert!(size_of::<OwnedPtr<()>>() == size_of::<*const ()>());
    assert!(align_of::<OwnedPtr<()>>() == align_of::<*const ()>());
    assert!(size_of::<OwnedPtr<dyn std::any::Any>>() == size_of::<*const dyn std::any::Any>());
    assert!(align_of::<OwnedPtr<dyn std::any::Any>>() == align_of::<*const dyn std::any::Any>());
    assert!(size_of::<Option<OwnedPtr<()>>>() == size_of::<OwnedPtr<()>>());
    assert!(align_of::<Option<OwnedPtr<()>>>() == align_of::<OwnedPtr<()>>());
};

/// `OwnedPtr` pointers are `Send` if `T` is `Send` because the data they
/// reference is unaliased. Note that this aliasing invariant is
/// unenforced by the type system; the abstraction using the
/// `OwnedPtr` must enforce it.
unsafe impl<T: Send + PointeeSized> Send for OwnedPtr<T> {}

/// `OwnedPtr` pointers are `Sync` if `T` is `Sync` because the data they
/// reference is unaliased. Note that this aliasing invariant is
/// unenforced by the type system; the abstraction using the
/// `OwnedPtr` must enforce it.
unsafe impl<T: Sync + PointeeSized> Sync for OwnedPtr<T> {}

impl<T: Sized> OwnedPtr<T> {
    /// Creates a new `OwnedPtr` that is dangling, but well-aligned.
    ///
    /// This is useful for initializing types which lazily allocate, like
    /// `Vec::new` does.
    ///
    /// Note that the address of the returned pointer may potentially
    /// be that of a valid pointer, which means this must not be used
    /// as a "not yet initialized" sentinel value.
    /// Types that lazily allocate must track initialization by some other means.
    #[must_use]
    #[inline]
    pub const fn dangling() -> Self {
        // FIXME(const-hack) replace with `From`
        OwnedPtr {
            pointer: NonNull::dangling(),
            _marker: PhantomData,
        }
    }
}

impl<T: PointeeSized> OwnedPtr<T> {
    /// Creates a new `OwnedPtr`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null.
    #[inline]
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        // SAFETY: the caller must guarantee that `ptr` is non-null.
        unsafe {
            OwnedPtr {
                pointer: NonNull::new_unchecked(ptr),
                _marker: PhantomData,
            }
        }
    }

    /// Creates a new `OwnedPtr` if `ptr` is non-null.
    #[inline]
    pub const fn new(ptr: *mut T) -> Option<Self> {
        if let Some(pointer) = NonNull::new(ptr) {
            Some(OwnedPtr {
                pointer,
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    /// Create a new `OwnedPtr` from a `NonNull` in const context.
    #[inline]
    pub const fn from_non_null(pointer: NonNull<T>) -> Self {
        OwnedPtr {
            pointer,
            _marker: PhantomData,
        }
    }

    /// Acquires the underlying `*mut` pointer.
    #[must_use = "`self` will be dropped if the result is not used"]
    #[inline]
    pub const fn as_ptr(self) -> *mut T {
        self.pointer.as_ptr()
    }

    /// Acquires the underlying `*mut` pointer.
    #[must_use = "`self` will be dropped if the result is not used"]
    #[inline]
    pub const fn as_non_null_ptr(self) -> NonNull<T> {
        self.pointer
    }

    /// Dereferences the content.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use `&*my_ptr.as_ptr()`.
    #[must_use]
    #[inline]
    pub const unsafe fn as_ref(&self) -> &T {
        // SAFETY: the caller must guarantee that `self` meets all the
        // requirements for a reference.
        unsafe { self.pointer.as_ref() }
    }

    /// Mutably dereferences the content.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use `&mut *my_ptr.as_ptr()`.
    #[must_use]
    #[inline]
    pub const unsafe fn as_mut(&mut self) -> &mut T {
        // SAFETY: the caller must guarantee that `self` meets all the
        // requirements for a mutable reference.
        unsafe { self.pointer.as_mut() }
    }

    /// Casts to a pointer of another type.
    #[must_use = "`self` will be dropped if the result is not used"]
    #[inline]
    pub const fn cast<U>(self) -> OwnedPtr<U> {
        // FIXME(const-hack): replace with `From`
        // SAFETY: is `NonNull`
        OwnedPtr {
            pointer: self.pointer.cast(),
            _marker: PhantomData,
        }
    }
}

impl<T: PointeeSized> Clone for OwnedPtr<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: PointeeSized> Copy for OwnedPtr<T> {}

unsafe impl<T: PointeeSized> TrivialClone for OwnedPtr<T> {}

impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<OwnedPtr<U>> for OwnedPtr<T> where T: Unsize<U> {}

impl<T: PointeeSized, U: PointeeSized> DispatchFromDyn<OwnedPtr<U>> for OwnedPtr<T> where
    T: Unsize<U>
{
}

impl<T: PointeeSized> fmt::Debug for OwnedPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: PointeeSized> fmt::Pointer for OwnedPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: PointeeSized> const From<&mut T> for OwnedPtr<T> {
    /// Converts a `&mut T` to a `OwnedPtr<T>`.
    ///
    /// This conversion is infallible since references cannot be null.
    #[inline]
    fn from(reference: &mut T) -> Self {
        Self::from(NonNull::from(reference))
    }
}

impl<T: PointeeSized> const From<NonNull<T>> for OwnedPtr<T> {
    /// Converts a `NonNull<T>` to a `OwnedPtr<T>`.
    ///
    /// This conversion is infallible since `NonNull` cannot be null.
    #[inline]
    fn from(pointer: NonNull<T>) -> Self {
        OwnedPtr::from_non_null(pointer)
    }
}

impl<T: PointeeSized> const From<OwnedPtr<T>> for NonNull<T> {
    #[inline]
    fn from(unique: OwnedPtr<T>) -> Self {
        unique.as_non_null_ptr()
    }
}
