use std::{
    borrow::{Borrow, BorrowMut},
    io::Cursor,
    ops::{Deref, DerefMut},
};

use crate::traits::StringRef;

#[derive(Debug)]
#[repr(transparent)]
pub struct Section {
    bytes: [u8],
}

#[derive(Debug, Clone)]
pub struct SectionBuilder {
    bytes: Vec<u8>,
}

#[repr(transparent)]
pub struct StringSection(pub Section);

impl StringSection {
    pub const fn get_string(&self, string_ref: StringRef) -> Option<&str> {
        #[inline(always)] // It is only called once
        fn rt_find_pos(bytes: &[u8]) -> Option<usize> {
            memchr::memchr(0, bytes)
        }
        #[inline(always)] // It is only called once
        const fn ct_find_pos(bytes: &[u8]) -> Option<usize> {
            let mut index = 0;
            while let Some(x) = bytes.get(index) {
                if *x == 0 {
                    return Some(index);
                }
                index += 1;
            }
            None
        }

        let bytes = self.0.as_bytes().get((string_ref.0 as usize)..)?;
        let end = core::intrinsics::const_eval_select((bytes,), ct_find_pos, rt_find_pos);
        match end {
            Some(x) => str::from_utf8(&bytes[..x]).ok(),
            None => str::from_utf8(bytes).ok(),
        }
    }
}

#[repr(transparent)]
pub struct StringSectionBuilder(pub SectionBuilder);

impl StringSectionBuilder {
    pub fn add_string(&mut self, s: &str) -> StringRef {
        let mut ind = 0;
        for bytes in self.0.as_bytes_mut().split(|x| *x == 0) {
            if bytes == s.as_bytes() {
                return StringRef(ind as u64);
            }
            ind += bytes.len() + 1;
        }
        let pos = self.0.len();
        self.0.extend_from_slice(s.as_bytes());
        self.0.push(0);
        StringRef(pos)
    }
}

impl Section {
    pub const fn new() -> &'static Self {
        const DATA: &Section = unsafe { &*std::ptr::from_raw_parts(std::ptr::dangling::<u8>(), 0) };
        &DATA
    }
    pub const fn new_mut() -> &'static mut Self {
        const DATA: &mut Section =
            unsafe { &mut *std::ptr::from_raw_parts_mut(std::ptr::dangling_mut::<u8>(), 0) };
        &mut *DATA
    }
    pub const fn with_bytes(bytes: &[u8]) -> &Self {
        unsafe { &*std::ptr::from_raw_parts(bytes.as_ptr(), bytes.len()) }
    }
    pub const fn with_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *std::ptr::from_raw_parts_mut(bytes.as_mut_ptr(), bytes.len()) }
    }

    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> u64 {
        self.bytes.len() as u64
    }
    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    #[inline(always)]
    pub const fn as_string_section(&self) -> &StringSection {
        unsafe { &*(self as *const Self as *const StringSection) }
    }
    #[inline(always)]
    pub const fn as_string_section_mut(&mut self) -> &mut StringSection {
        unsafe { &mut *(self as *mut Self as *mut StringSection) }
    }
}

impl SectionBuilder {
    pub const fn new() -> Self {
        Self::with_bytes(Vec::new())
    }
    pub const fn with_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> u64 {
        self.bytes.len() as u64
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    pub fn push(&mut self, v: u8) {
        self.bytes.push(v);
    }
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.bytes.extend_from_slice(slice);
    }
    pub fn construct_mut_vec_cursor(&mut self) -> Cursor<&mut Vec<u8>> {
        Cursor::new(&mut self.bytes)
    }

    #[inline(always)]
    pub fn as_string_section(&self) -> &StringSectionBuilder {
        unsafe { &*(self as *const Self as *const StringSectionBuilder) }
    }
    #[inline(always)]
    pub fn as_string_section_mut(&mut self) -> &mut StringSectionBuilder {
        unsafe { &mut *(self as *mut Self as *mut StringSectionBuilder) }
    }
}

impl AsRef<[u8]> for Section {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToOwned for Section {
    type Owned = SectionBuilder;

    fn to_owned(&self) -> Self::Owned {
        Self::Owned {
            bytes: self.bytes.to_owned(),
        }
    }

    fn clone_into(&self, target: &mut Self::Owned) {
        self.bytes.clone_into(&mut target.bytes);
    }
}

impl AsRef<[u8]> for SectionBuilder {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Borrow<Section> for SectionBuilder {
    fn borrow(&self) -> &Section {
        Section::with_bytes(&self.bytes)
    }
}

impl BorrowMut<Section> for SectionBuilder {
    fn borrow_mut(&mut self) -> &mut Section {
        Section::with_bytes_mut(&mut self.bytes)
    }
}

impl Deref for SectionBuilder {
    type Target = Section;
    fn deref(&self) -> &Self::Target {
        Section::with_bytes(&self.bytes)
    }
}

impl DerefMut for SectionBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Section::with_bytes_mut(&mut self.bytes)
    }
}

impl Deref for StringSectionBuilder {
    type Target = StringSection;
    fn deref(&self) -> &Self::Target {
        Section::with_bytes(&self.0.bytes).as_string_section()
    }
}

impl DerefMut for StringSectionBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Section::with_bytes_mut(&mut self.0.bytes).as_string_section_mut()
    }
}

impl Borrow<StringSection> for StringSectionBuilder {
    fn borrow(&self) -> &StringSection {
        &**self
    }
}

impl BorrowMut<StringSection> for StringSectionBuilder {
    fn borrow_mut(&mut self) -> &mut StringSection {
        &mut **self
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SectionInfo {
    pub offset: u64,
    pub len: u64,
}

impl SectionInfo {
    pub const fn new(offset: u64, len: u64) -> Self {
        Self { offset, len }
    }
    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { &*std::ptr::from_raw_parts(self, size_of::<Self>()) }
    }
}
