#[inline(always)]
pub fn clone_utf16str(this: &Box<widestring::Utf16Str>) -> Box<widestring::Utf16Str> {
    unsafe {
        widestring::Utf16Str::from_boxed_slice_unchecked(Box::clone_from_ref(this.as_slice()))
    }
}
