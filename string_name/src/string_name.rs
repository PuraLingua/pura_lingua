#![feature(pattern)]

use derive_more::Deref;
use faststr::FastStr;
use global_proc_macros::ThreadSafe;
use std::borrow::Borrow;
use std::ops::Add;
use std::path::Path;
use std::sync::Arc;
use std::{
    fmt::{Debug, Display},
    str::{FromStr, pattern::Pattern},
};

#[repr(transparent)]
#[derive(Clone, Eq, Hash, Deref, Default, ThreadSafe)]
pub struct StringName {
    s: FastStr,
}

impl StringName {
    pub const fn from_static_str(s: &'static str) -> Self {
        Self {
            s: FastStr::from_static_str(s),
        }
    }
    pub fn as_str(&self) -> &str {
        self.s.as_str()
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.s.as_bytes().to_vec()
    }
    pub fn from_arc_str(s: Arc<str>) -> Self {
        Self {
            s: FastStr::from_arc_str(s),
        }
    }
    pub fn from_arc_string(s: Arc<String>) -> Self {
        Self {
            s: FastStr::from_arc_string(s),
        }
    }
    pub fn from_string(s: String) -> Self {
        Self {
            s: FastStr::from_string(s),
        }
    }
}

impl StringName {
    pub fn contains<P: Pattern>(&self, pat: P) -> bool {
        self.s.contains(pat)
    }
}

impl Debug for StringName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <FastStr as Debug>::fmt(&self.s, f)
    }
}

impl Display for StringName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <FastStr as Display>::fmt(&self.s, f)
    }
}

impl Borrow<str> for StringName {
    fn borrow(&self) -> &str {
        self.s.as_str()
    }
}

impl PartialEq<str> for StringName {
    fn eq(&self, other: &str) -> bool {
        self.s.as_str().eq(other)
    }
}

impl<T: AsRef<str>> PartialEq<T> for StringName {
    fn eq(&self, other: &T) -> bool {
        self.eq(other.as_ref())
    }
}

impl From<&str> for StringName {
    fn from(value: &str) -> Self {
        Self {
            s: FastStr::from_str(value).unwrap(),
        }
    }
}

impl From<String> for StringName {
    fn from(value: String) -> Self {
        Self {
            s: FastStr::from_string(value),
        }
    }
}
impl<T: AsRef<str>> Add<T> for StringName {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.as_ref();
        Self {
            s: FastStr::from_string(self.s.to_string() + rhs),
        }
    }
}

impl AsRef<Path> for StringName {
    fn as_ref(&self) -> &Path {
        <Self as Borrow<str>>::borrow(self).as_ref()
    }
}

impl AsRef<str> for StringName {
    fn as_ref(&self) -> &str {
        self.borrow()
    }
}

impl AsRef<StringName> for StringName {
    fn as_ref(&self) -> &StringName {
        self
    }
}
