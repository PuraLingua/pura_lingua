#![feature(decl_macro)]
#![feature(ptr_as_ref_unchecked)]
#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(const_try)]
#![feature(const_format_args)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(const_cmp)]
#![feature(const_range)]
#![feature(const_ops)]

use std::io::{Cursor, Read, Seek, Write};

use global_errors::{BinaryError, GenericError};
use indexmap::IndexSet;

pub mod integer;
pub mod item_token;

#[derive(Debug, Clone, Default)]
pub struct File {
    pub interner: StringInterner,
    pub data: Cursor<Vec<u8>>,
}

impl File {
    pub fn writer(&mut self) -> &mut (impl Write + Seek) {
        &mut self.data
    }
    pub fn reader(&mut self) -> &mut (impl Read + Seek) {
        &mut self.data
    }

    pub fn string_position_of(
        &mut self,
        s: &str,
    ) -> global_errors::Result<u64, GenericError<BinaryError>> {
        self.interner.position_of(s)
    }
    pub fn get_string(&self, i: u64) -> global_errors::Result<&str, GenericError<BinaryError>> {
        self.interner.get(i)
    }
}

#[derive(Debug, Clone)]
pub struct StringInterner {
    pub set: IndexSet<String>,
}

impl Default for StringInterner {
    fn default() -> Self {
        let mut set = IndexSet::new();
        set.insert(String::new());
        Self { set }
    }
}

impl StringInterner {
    pub fn position_of(
        &mut self,
        s: &str,
    ) -> global_errors::Result<u64, GenericError<BinaryError>> {
        match self.set.iter().position(|x| x.eq(s)) {
            Some(pos) => Ok(pos as u64),
            None => {
                let l = self.set.len();
                self.set.insert(s.to_owned());
                Ok(l as u64)
            }
        }
    }
    pub fn get(&self, i: u64) -> global_errors::Result<&str, GenericError<BinaryError>> {
        let mut x = i as usize;
        for s in self.set.iter() {
            if x == 0 {
                return Ok(s);
            }
            x -= 1;
        }
        Err(BinaryError::StringNotFound { index: i }.throw())
    }
}
