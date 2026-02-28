use std::{
    alloc::{AllocError, Allocator, Layout},
    clone::CloneToUninit,
    mem::MaybeUninit,
    ptr::{Alignment, NonNull},
    sync::RwLock,
};

use bon::Builder;

#[cfg(test)]
mod tests;

#[derive(Builder, Clone, Copy)]
#[builder(const)]
pub struct GetLayoutOptions {
    pub(crate) prefer_cached: bool,
    pub(crate) discard_calculated_layout: bool,
}

impl const Default for GetLayoutOptions {
    fn default() -> Self {
        Self {
            prefer_cached: true,
            discard_calculated_layout: false,
        }
    }
}

#[derive(Builder, Clone, Copy)]
#[builder(const)]
pub struct GetFieldOffsetOptions {
    pub(crate) prefer_cached: bool,
    pub(crate) discard_calculated_offset: bool,
}

impl const Default for GetFieldOffsetOptions {
    fn default() -> Self {
        Self {
            prefer_cached: true,
            discard_calculated_offset: false,
        }
    }
}

pub fn alloc_type<T, A: Allocator>(allocator: &A) -> Result<NonNull<T>, AllocError> {
    allocator
        .allocate_zeroed(Layout::new::<T>())
        .map(|x| x.cast())
}

/// On arithmetic overflow or when the total size would exceed
/// [`isize::MAX`], returns [`None`].
///
/// Copy from [`Layout::array`]
pub const fn arrayed_layout(element_layout: Layout, len: usize) -> Option<Layout> {
    let element_size = element_layout.size();
    let align = unsafe { Alignment::new_unchecked(element_layout.align()) };

    /* cSpell:disable */
    #[inline(always)]
    const fn max_size_for_align(align: Alignment) -> usize {
        // (power-of-two implies align != 0.)

        // Rounded up size is:
        //   size_rounded_up = (size + align - 1) & !(align - 1);
        //
        // We know from above that align != 0. If adding (align - 1)
        // does not overflow, then rounding up will be fine.
        //
        // Conversely, &-masking with !(align - 1) will subtract off
        // only low-order-bits. Thus if overflow occurs with the sum,
        // the &-mask cannot subtract enough to undo that overflow.
        //
        // Above implies that checking for summation overflow is both
        // necessary and sufficient.

        // SAFETY: the maximum possible alignment is `isize::MAX + 1`,
        // so the subtraction cannot overflow.
        unsafe { std::intrinsics::unchecked_sub(isize::MAX as usize + 1, align.as_usize()) }
    }

    #[inline(always)]
    const fn check_len(len: usize, element_size: usize, align: Alignment) -> bool {
        match len.checked_mul(element_size) {
            Some(x) => x > max_size_for_align(align),
            None => false,
        }
    }

    // We need to check two things about the size:
    //  - That the total size won't overflow a `usize`, and
    //  - That the total size still fits in an `isize`.
    // By using division we can check them both with a single threshold.
    // That'd usually be a bad idea, but thankfully here the element size
    // and alignment are constants, so the compiler will fold all of it.
    if element_size != 0 && check_len(len, element_size, align) {
        return None;
    }

    // SAFETY: We just checked that we won't overflow `usize` when we multiply.
    // This is a useless hint inside this function, but after inlining this helps
    // deduplicate checks for whether the overall capacity is zero (e.g., in RawVec's
    // allocation path) before/after this multiplication.
    let array_size = unsafe { std::intrinsics::unchecked_mul(element_size, len) };

    // SAFETY: We just checked above that the `array_size` will not
    // exceed `isize::MAX` even when rounded up to the alignment.
    // And `Alignment` guarantees it's a power of two.
    unsafe { Some(Layout::from_size_alignment_unchecked(array_size, align)) }
    /* cSpell:enable */
}

#[inline(always)]
pub const fn get_return_layout_for_libffi(layout: Layout) -> Layout {
    if layout.size() < size_of::<usize>() {
        Layout::new::<usize>()
    } else {
        layout
    }
}

#[derive(Clone, Copy)]
pub struct GuardedBox<'a, T: ?Sized, A: Allocator>(NonNull<T>, &'a AllocateGuard<A>);

impl<'a, T: ?Sized, A: Allocator> GuardedBox<'a, T, A> {
    pub fn new(val: T, guard: &'a AllocateGuard<A>) -> Self
    where
        T: Sized,
    {
        let mut this = Self::new_uninit(guard);
        GuardedBox::write(&mut this, val);
        unsafe { GuardedBox::assume_init(this) }
    }
    pub fn new_uninit(guard: &'a AllocateGuard<A>) -> GuardedBox<'a, MaybeUninit<T>, A>
    where
        T: Sized,
    {
        let layout = Layout::new::<T>();
        match GuardedBox::try_new_uninit(guard) {
            Ok(data) => data,
            Err(_) => std::alloc::handle_alloc_error(layout),
        }
    }
    pub fn try_new_uninit(
        guard: &'a AllocateGuard<A>,
    ) -> Result<GuardedBox<'a, MaybeUninit<T>, A>, AllocError>
    where
        T: Sized,
    {
        guard
            .allocate(Layout::new::<MaybeUninit<T>>())
            .map(|x| GuardedBox(x.as_non_null_ptr().cast(), guard))
    }
    pub const fn write(b: &mut GuardedBox<'a, MaybeUninit<T>, A>, val: T)
    where
        T: Sized,
    {
        unsafe {
            b.0.as_mut().write(val);
        }
    }
    pub const unsafe fn assume_init(b: GuardedBox<'a, MaybeUninit<T>, A>) -> Self
    where
        T: Sized,
    {
        Self(b.0.cast(), b.1)
    }

    pub fn clone_from_ref(val: &T, guard: &'a AllocateGuard<A>) -> Self
    where
        T: CloneToUninit,
    {
        let (ptr, _) = Box::into_non_null_with_allocator(Box::clone_from_ref_in(val, guard));
        Self(ptr, guard)
    }
    pub const fn as_non_null(b: GuardedBox<'a, T, A>) -> NonNull<T> {
        b.0
    }
}

pub struct AllocateGuard<A: Allocator> {
    a: A,
    record: RwLock<Vec<(NonNull<u8>, Layout)>>,
}

impl<A: Allocator + [const] Default> const Default for AllocateGuard<A> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<A: Allocator> AllocateGuard<A> {
    pub const fn new(a: A) -> Self {
        Self {
            a,
            record: RwLock::new(Vec::new()),
        }
    }
}

impl AllocateGuard<std::alloc::Global> {
    pub const fn global() -> Self {
        Self::new(std::alloc::Global)
    }
}

impl AllocateGuard<std::alloc::System> {
    pub const fn system() -> Self {
        Self::new(std::alloc::System)
    }
}

unsafe impl<A: Allocator> Allocator for AllocateGuard<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.a.allocate(layout)?;
        let mut record = self.record.write().unwrap();
        record.push((ptr.as_non_null_ptr(), layout));
        Ok(ptr)
    }

    /// Do not call it as everything will be dropped when self is dropped.
    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {}

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.a.allocate_zeroed(layout)?;
        let mut record = self.record.write().unwrap();
        record.push((ptr.as_non_null_ptr(), layout));
        Ok(ptr)
    }

    /// It will not deallocate ptr
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            std::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
        }

        Ok(new_ptr)
    }

    /// It will not deallocate ptr
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate_zeroed(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            std::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
        }

        Ok(new_ptr)
    }

    /// It will not deallocate ptr
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be lower than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `new_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            std::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), new_layout.size());
        }

        Ok(new_ptr)
    }
}

impl<A: Allocator> Drop for AllocateGuard<A> {
    fn drop(&mut self) {
        let record = self.record.read().unwrap();
        for (p, layout) in &*record {
            unsafe {
                self.a.deallocate(*p, *layout);
            }
        }
    }
}
