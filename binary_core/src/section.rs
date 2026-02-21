use std::io::Cursor;

use crate::traits::StringRef;

#[derive(Debug, Clone)]
pub struct Section {
    bytes: Vec<u8>,
}

#[repr(transparent)]
pub struct StringSection(pub Section);

impl StringSection {
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

    pub fn get_string(&self, string_ref: StringRef) -> Option<&str> {
        let bytes = self.0.as_bytes().get((string_ref.0 as usize)..)?;
        match bytes.split_once(|&x| x == 0) {
            Some((res, _)) => str::from_utf8(res).ok(),
            None => str::from_utf8(bytes).ok(),
        }
    }
}

impl const Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

impl Section {
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
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
    pub fn construct_mut_vec_cursor(&mut self) -> Cursor<&mut Vec<u8>> {
        Cursor::new(&mut self.bytes)
    }
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.bytes.extend_from_slice(slice);
    }
    pub fn push(&mut self, byte: u8) {
        self.bytes.push(byte);
    }
    #[inline(always)]
    pub fn as_string_section(&self) -> &StringSection {
        unsafe { &*(self as *const Self as *const StringSection) }
    }
    #[inline(always)]
    pub fn as_string_section_mut(&mut self) -> &mut StringSection {
        unsafe { &mut *(self as *mut Self as *mut StringSection) }
    }
}

impl AsRef<[u8]> for Section {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
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
