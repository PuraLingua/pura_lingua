#![feature(decl_macro)]
#![feature(const_default)]
#![feature(const_trait_impl)]

use std::panic::Location;

use derive_more::Display;

use string_name::StringName;

pub enum TypeSystemError {}

#[derive(Clone, Debug, Display, thiserror::Error)]
pub enum RuntimeError {
    /// Negative if the register number is unknown
    #[display("FailedGetRegister({_0})")]
    FailedGetRegister(i64),
    /// Negative if the register number is unknown
    #[display("FailedReadRegister({_0})")]
    FailedReadRegister(i64),
    /// Negative if the register number is unknown
    #[display("FailedWriteRegister({_0})")]
    FailedWriteRegister(i64),
    #[display("FailedGetAssembly({_0})")]
    FailedGetAssembly(StringName),
    #[display("FailedGetField({_0})")]
    FailedGetField(u32),
    #[display("FailedSetField({_0})")]
    FailedSetField(u32),
    FailedMakeGeneric,
    GenericTypeNotInitializedYet,
    ResolutionNotCompleted,

    UnsupportedEntryType,
    UnsupportedInstanceType,
    UnsupportedObjectType,
    UnsupportedAttributeType,
    UnsupportedGettingField,
    UnsupportedParentType,

    NonArrayLike,

    MethodReturnsAbnormally,

    ValueParsed,

    NonGenericType(StringName),

    WrongType,

    UninitializedMethodTable,

    BrokenReference,
    ConstructStaticClass,
    #[display("UnexpectedCanon({_0})")]
    UnexpectedCanon(
        /// The name of type
        StringName,
    ),
    NoConsole,
    #[cfg(windows)]
    WindowsAPIError,
    #[cfg(windows)]
    InvalidConsoleColor,
    #[cfg(unix)]
    LibcError(std::ffi::c_int),
    #[display(
        "{}",
        if *is_width {
            "ConsoleWidthBufferLessThanWindowSize"
        } else {
            "ConsoleHeightBufferLessThanWindowSize"
        }
    )]
    ConsoleBufferLessThanWindowSize {
        /// System_Console_ buffer width is less than window size if `true`, or height is less than window size
        is_width: bool,
    },
}

impl RuntimeError {
    #[track_caller]
    pub fn throw(self) -> GenericError<RuntimeError> {
        GenericError::throw(self)
    }
}

#[derive(Clone, Copy, Debug, Display, thiserror::Error)]
pub struct UnwrapError;

#[derive(thiserror::Error, Display, Debug)]
pub enum BinaryError {
    StringNotFound {
        index: u64,
    },
    IndexOutOfRange,
    IntOutOfRange,
    #[display("Unexpected `TypeSpecificAttr`: {_0}")]
    UnexpectedTypeSpecificAttr(&'static str),
    WrongFileFormat,
    SectionNotFound,
    BinaryTooShort,
    EnumOutOfBounds(&'static str),
    #[display(
        "MergeAssembliesWithDifferentName {{self_name: {self_name}, other_name: {other_name}}}"
    )]
    MergeAssembliesWithDifferentName {
        self_name: StringName,
        other_name: StringName,
    },
    #[display("MergeConflict({_0})")]
    MergeConflict(StringName),
}

impl BinaryError {
    #[track_caller]
    #[inline(always)]
    pub fn throw(self) -> GenericError<Self> {
        GenericError::throw(self)
    }
}

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

#[derive(Clone, Debug, Display, thiserror::Error)]
pub enum EncodingError {
    UnsupportedEncoding(&'static str),
}

pub use anyhow::anyhow;

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
