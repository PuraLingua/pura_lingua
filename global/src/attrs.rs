use std::fmt::Debug;

use binary_proc_macros::{ReadFromFile, WriteToFile};
use derive_ctor::ctor;
use enumflags2::{BitFlags, bitflags, make_bitflags};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::{UnwrapEnum, WithType};

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FieldImplementationFlags {
    Static,
}

#[derive(Clone, Copy, Debug, ctor, CopyGetters, Setters, MutGetters, ReadFromFile, WriteToFile)]
#[ctor(pub new)]
#[getset(set = "pub", get_mut = "pub")]
#[get_copy = "pub"]
pub struct FieldAttr {
    vis: Visibility,
    impl_flags: BitFlags<FieldImplementationFlags>,
}

impl FieldAttr {
    pub fn is_static(&self) -> bool {
        self.impl_flags.contains(FieldImplementationFlags::Static)
    }
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MethodImplementationFlags {
    Static,
    ImplementedByRuntime,
    HideWhenCapturing,
}

#[derive(
    Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, Eq, PartialEq, ReadFromFile, WriteToFile,
)]
#[derive_const(Default)]
#[repr(u8)]
pub enum CallConvention {
    /// i.e. extern "system"
    #[default]
    PlatformDefault,
    CDecl,
    CDeclWithVararg,
    Win64,
    SystemV,
    Stdcall,
    Fastcall,
}

impl std::fmt::Display for CallConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlatformDefault => write!(f, "system"),
            Self::CDecl => write!(f, "C"),
            Self::CDeclWithVararg => write!(f, "C"),
            Self::Win64 => write!(f, "win64"),
            /* cSpell:disable-next-line */
            Self::SystemV => write!(f, "sysv64"),
            Self::Stdcall => write!(f, "stdcall"),
            Self::Fastcall => write!(f, "fastcall"),
        }
    }
}

#[derive(
    Clone, CopyGetters, Debug, ctor, Setters, MutGetters, Getters, ReadFromFile, WriteToFile,
)]
#[ctor(pub new)]
#[getset(set = "pub", get_mut = "pub")]
#[get_copy = "pub"]
pub struct MethodAttr<TType> {
    vis: Visibility,
    impl_flags: BitFlags<MethodImplementationFlags>,
    #[getset(skip)]
    #[get = "pub"]
    overrides: Option<u32>,
    #[getset(skip)]
    #[get = "pub"]
    local_variable_types: Vec<TType>,
}

impl<TType> MethodAttr<TType> {
    pub fn for_sctor(local_variable_types: Vec<TType>) -> Self {
        Self {
            vis: Visibility::Public,
            impl_flags: make_bitflags!(MethodImplementationFlags::{Static}),
            overrides: None,
            local_variable_types,
        }
    }
    pub fn is_static(&self) -> bool {
        self.impl_flags()
            .contains(MethodImplementationFlags::Static)
    }
    pub fn map_types<_TType, F>(self, f: F) -> MethodAttr<_TType>
    where
        F: Fn(TType) -> _TType,
    {
        MethodAttr {
            vis: self.vis,
            impl_flags: self.impl_flags,
            overrides: self.overrides,
            local_variable_types: self.local_variable_types.into_iter().map(f).collect(),
        }
    }
}

#[derive(Clone, Copy, CopyGetters, Debug, ctor, Setters, Getters, ReadFromFile, WriteToFile)]
#[ctor(pub new)]
#[getset(set = "pub", get_mut = "pub")]
#[get_copy = "pub"]
pub struct ParameterAttr {
    impl_flags: BitFlags<ParameterImplementationFlags>,
}

impl ParameterAttr {
    pub fn is_by_ref(&self) -> bool {
        self.impl_flags
            .contains(ParameterImplementationFlags::ByRef)
    }
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, derive_more::Display)]
pub enum ParameterImplementationFlags {
    ByRef,
}

#[derive(
    Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, Eq, PartialEq, ReadFromFile, WriteToFile,
)]
#[repr(u8)]
pub enum Visibility {
    Public,
    Private,
    AssemblyOnly,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, UnwrapEnum, WithType, Eq, PartialEq, ReadFromFile, WriteToFile)]
#[with_type(derive = (TryFromPrimitive, IntoPrimitive, Clone, Copy, ReadFromFile, WriteToFile))]
#[unwrap_enum(ref, ref_mut)]
pub enum TypeSpecificAttr {
    Class(BitFlags<ClassImplementationFlags>),
    Struct(BitFlags<StructImplementationFlags>),
    Interface(BitFlags<InterfaceImplementationFlags>),
}

impl TypeSpecificAttr {
    pub fn is_partial(&self) -> bool {
        match self {
            TypeSpecificAttr::Class(flags) => flags.contains(ClassImplementationFlags::Partial),
            TypeSpecificAttr::Struct(flags) => flags.contains(StructImplementationFlags::Partial),
            TypeSpecificAttr::Interface(flags) => {
                flags.contains(InterfaceImplementationFlags::Partial)
            }
        }
    }
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StructImplementationFlags {
    Ref,
    Partial,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassImplementationFlags {
    Static,
    Partial,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterfaceImplementationFlags {
    Partial,
}

#[repr(C)]
#[derive(
    Clone,
    Copy,
    Debug,
    ctor,
    CopyGetters,
    Setters,
    MutGetters,
    Eq,
    PartialEq,
    ReadFromFile,
    WriteToFile,
)]
#[ctor(pub new)]
#[getset(set = "pub", get_mut = "pub")]
#[get_copy = "pub"]
pub struct TypeAttr {
    vis: Visibility,
    specific: TypeSpecificAttr,
}
