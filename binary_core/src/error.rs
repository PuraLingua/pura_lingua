use std::{
    fmt::Display,
    io::{ErrorKind, const_error},
};

use crate::traits::StringRef;

#[derive(Debug)]
pub enum Error {
    WrongFileSize,
    WrongFormat,
    IntegerOutOfRange,
    IndexOutOfRange,
    InvalidChar,
    InheritFromGeneric,
    WrongParentType,
    UnknownSection(usize),
    UnknownStringRef(StringRef),
    IoError(std::io::Error),
    EnumOutOfBounds(&'static str),
    Custom(global_errors::Error),
}

pub type BinaryResult<T> = Result<T, Error>;

impl const From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl const From<global_errors::Error> for Error {
    fn from(value: global_errors::Error) -> Self {
        Self::Custom(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongFileSize => f.write_str("WrongFileSize"),
            Self::WrongFormat => f.write_str("WrongFormat"),
            Self::IntegerOutOfRange => f.write_str("IntegerOutOfRange"),
            Self::IndexOutOfRange => f.write_str("IndexOutOfRange"),
            Self::InvalidChar => f.write_str("InvalidChar"),
            Self::InheritFromGeneric => f.write_str("InheritFromGeneric"),
            Self::WrongParentType => f.write_str("WrongParentType"),
            Self::UnknownSection(id) => f.write_fmt(format_args!("UnknownSection: {id}")),
            Self::UnknownStringRef(s) => f.write_fmt(format_args!("UnknownStringRef: {s:?}")),
            Self::IoError(error) => <_ as Display>::fmt(error, f),
            Self::EnumOutOfBounds(ty) => f.write_fmt(format_args!("EnumOutOfBounds: {ty}")),
            Self::Custom(error) => <_ as Display>::fmt(error, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl Error {
    pub const fn unwrap_io(self) -> std::io::Error {
        const fn const_unwrap(this: Error) -> std::io::Error {
            if !matches!(&this, Error::IoError(_)) {
                panic!();
            }
            const OFFSET: usize = std::mem::offset_of!(Error, IoError.0);
            let this = unsafe { std::mem::transmute::<Error, [u8; size_of::<Error>()]>(this) };
            unsafe {
                std::mem::transmute_copy::<_, std::io::Error>(
                    this[OFFSET..]
                        .as_array::<{ size_of::<std::io::Error>() }>()
                        .unwrap(),
                )
            }
        }
        fn unwrap(this: Error) -> std::io::Error {
            match this {
                Error::IoError(error) => error,
                _ => panic!(),
            }
        }
        std::intrinsics::const_eval_select((self,), const_unwrap, unwrap)
    }
    pub const fn enum_out_of_bounds<TEnum: ?Sized>() -> Self {
        Self::EnumOutOfBounds(std::any::type_name::<TEnum>())
    }
}

/// Common errors constants for use in std
/// Copied from [`std::io::Error`]
#[allow(dead_code)]
impl Error {
    pub(crate) const INVALID_UTF8: Self =
        const_error!(ErrorKind::InvalidData, "stream did not contain valid UTF-8").into();

    pub(crate) const READ_EXACT_EOF: Self =
        const_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer").into();

    pub(crate) const UNKNOWN_THREAD_COUNT: Self = const_error!(
        ErrorKind::NotFound,
        "the number of hardware threads is not known for the target platform",
    )
    .into();

    pub(crate) const UNSUPPORTED_PLATFORM: Self = const_error!(
        ErrorKind::Unsupported,
        "operation not supported on this platform"
    )
    .into();

    pub(crate) const WRITE_ALL_EOF: Self =
        const_error!(ErrorKind::WriteZero, "failed to write whole buffer").into();

    pub(crate) const ZERO_TIMEOUT: Self =
        const_error!(ErrorKind::InvalidInput, "cannot set a 0 duration timeout").into();

    pub(crate) const NO_ADDRESSES: Self = const_error!(
        ErrorKind::InvalidInput,
        "could not resolve to any addresses"
    )
    .into();
}

#[cfg(test)]
#[doc(hidden)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "checked"]
    fn test_unwrap_io() {
        const fn const_unwrap(this: Error) -> std::io::Error {
            if !matches!(&this, Error::IoError(_)) {
                panic!();
            }
            const OFFSET: usize = std::mem::offset_of!(Error, IoError.0);
            let this = unsafe { std::mem::transmute::<Error, [u8; size_of::<Error>()]>(this) };
            unsafe {
                std::mem::transmute_copy::<_, std::io::Error>(
                    this[OFFSET..]
                        .as_array::<{ size_of::<std::io::Error>() }>()
                        .unwrap(),
                )
            }
        }
        fn unwrap(this: Error) -> std::io::Error {
            match this {
                Error::IoError(error) => error,
                _ => panic!(),
            }
        }
        macro t($($i:ident)*) {$(
            assert_eq!(dbg!(unwrap(Error::$i)).kind(), dbg!(const_unwrap(Error::$i)).kind());
        )*}
        t! {
            INVALID_UTF8
            READ_EXACT_EOF
            UNKNOWN_THREAD_COUNT
            UNSUPPORTED_PLATFORM
            WRITE_ALL_EOF
            ZERO_TIMEOUT
            NO_ADDRESSES
        }
    }
}
