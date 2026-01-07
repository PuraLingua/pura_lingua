use std::{
    alloc::{AllocError, Allocator, Layout},
    ptr::{Alignment, NonNull},
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
    allocator.allocate(Layout::new::<T>()).map(|x| x.cast())
}

/// Return None if overflowed
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

    // We need to check two things about the size:
    //  - That the total size won't overflow a `usize`, and
    //  - That the total size still fits in an `isize`.
    // By using division we can check them both with a single threshold.
    // That'd usually be a bad idea, but thankfully here the element size
    // and alignment are constants, so the compiler will fold all of it.
    if element_size != 0 && len > max_size_for_align(align) / element_size {
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
    unsafe {
        Some(Layout::from_size_align_unchecked(
            array_size,
            align.as_usize(),
        ))
    }
    /* cSpell:enable */
}
