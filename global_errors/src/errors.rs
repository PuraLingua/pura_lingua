#![feature(decl_macro)]
#![feature(const_default)]
#![feature(const_trait_impl)]

use std::panic::Location;

use derive_more::Display;

#[derive(Clone, Copy, Debug, Display, thiserror::Error)]
pub struct UnwrapError;

#[derive(Clone, Debug, Display)]
#[display("Error: {e:#?}\n(caller: {caller})")]
pub struct GenericError<E: std::error::Error + 'static> {
    e: E,
    caller: &'static Location<'static>,
}

impl<E: std::error::Error + 'static> GenericError<E> {
    #[track_caller]
    #[inline(always)]
    pub fn throw(e: E) -> Self {
        Self {
            e,
            caller: Location::caller(),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for GenericError<E> {}

pub use anyhow::{Error, anyhow};

macro single_error($($(#[$($meta:meta)*])* $i:ident @ $ctor:ident)*) {$(
    $(#[$($meta)*])*
    #[derive(Clone, Copy, Debug, ::derive_more::Display, ::thiserror::Error)]
    pub struct $i;

    impl const Default for $i {
        fn default() -> Self {
            Self::$ctor()
        }
    }

    impl $i {
        pub const fn $ctor() -> Self {
            Self
        }
    }
)*}

single_error! {
    ConstFromStrError@new
    NullPointerError@new
    /// It will be returned if a value of type
    /// [`std::option::Option`] is
    /// [`std::option::Option::None`]
    NoneError@new
    OutOfMemoryError@new
    PoisonError@new
}

#[derive(Clone, Copy, Debug, ::derive_more::Display, ::thiserror::Error)]
pub enum LayoutCalculateError {
    WhenArray,
}

#[derive(Clone, Copy, Debug, ::derive_more::Display, ::thiserror::Error)]
#[display(
    "
FatalError:
    Message: {_0}

    at {_1}
"
)]
pub struct FatalError(&'static str, &'static Location<'static>);

impl FatalError {
    #[track_caller]
    pub const fn new(msg: &'static str) -> Self {
        Self(msg, Location::caller())
    }
}

#[derive(Clone, Copy, Debug, ::derive_more::Display, ::thiserror::Error)]
#[display(
    "
IndexOutOfRange:
    Message: {msg}

    Index: {index}

    at {location}
"
)]
pub struct IndexOutOfRangeError {
    msg: &'static str,
    index: usize,
    location: &'static Location<'static>,
}

impl IndexOutOfRangeError {
    #[track_caller]
    pub const fn new(msg: &'static str, index: usize) -> Self {
        Self {
            msg,
            index,
            location: Location::caller(),
        }
    }
}

pub use anyhow::Result;
